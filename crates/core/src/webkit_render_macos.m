/*
 * webkit_render_macos.m — WebKit rendering helpers for macOS.
 *
 * Uses WKWebView (WebKit.framework) for both PDF and PNG output.
 * Compiled by the cc crate when the target is macOS.
 *
 * PDF:  WKWebView -printOperationWithPrintInfo: → NSPrintOperation
 * PNG:  WKWebView -takeSnapshotWithConfiguration:completionHandler:
 *
 * Requires macOS 11.0+ (Big Sur) for the PDF API.
 */

#import <WebKit/WebKit.h>
#import <AppKit/AppKit.h>
#include <stdlib.h>
#include <string.h>
#include "webkit_render.h"

/* ── Thread-local error buffer ───────────────────────────────────────────── */

static _Thread_local char wk_err_buf[2048] = {0};

const char *wk_last_error(void) {
    return wk_err_buf;
}

void wk_free(void *ptr) {
    free(ptr);
}

/* ── Navigation delegate ─────────────────────────────────────────────────── */

@interface WKRenderDelegate : NSObject <WKNavigationDelegate>
@property (atomic) BOOL loadDone;
@property (atomic) BOOL loadFailed;
@property (atomic, strong) NSError *loadError;
@end

@implementation WKRenderDelegate
{
    /* Make ivars package-visible so spin_runloop_until can
     * take a pointer to them from the same translation unit. */
    @public BOOL _loadDone;
    @public BOOL _loadFailed;
}

- (void)webView:(WKWebView *)webView
    didFinishNavigation:(WKNavigation *)navigation {
    self.loadDone = YES;
}

- (void)webView:(WKWebView *)webView
    didFailNavigation:(WKNavigation *)navigation
            withError:(NSError *)error {
    self.loadError = error;
    self.loadFailed = YES;
    self.loadDone = YES;
}

- (void)webView:(WKWebView *)webView
    didFailProvisionalNavigation:(WKNavigation *)navigation
                       withError:(NSError *)error {
    self.loadError = error;
    self.loadFailed = YES;
    self.loadDone = YES;
}

@end

/* ── Run-loop helper ─────────────────────────────────────────────────────── */

static void spin_runloop_until(BOOL *flag, NSTimeInterval timeout) {
    NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:timeout];
    while (!(*flag) && [deadline timeIntervalSinceNow] > 0) {
        [[NSRunLoop currentRunLoop]
            runMode:NSDefaultRunLoopMode
         beforeDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];
    }
}

/* ── URL helper ──────────────────────────────────────────────────────────── */

static NSURL *url_from_c_string(const char *url) {
    NSString *str = [NSString stringWithUTF8String:url];
    if ([str hasPrefix:@"http://"] || [str hasPrefix:@"https://"] ||
        [str hasPrefix:@"file://"]) {
        return [NSURL URLWithString:str];
    }
    /* Bare file path → file:// URL. */
    return [NSURL fileURLWithPath:str];
}

/* ── Load a URL into a WKWebView and wait for it to finish ───────────────  */

static int load_url(WKWebView *webView, WKRenderDelegate *delegate,
                    const char *url, int js_delay_ms) {
    NSURL *nsUrl = url_from_c_string(url);
    if (!nsUrl) {
        snprintf(wk_err_buf, sizeof(wk_err_buf), "invalid URL: %s", url);
        return WK_ERR_LOAD;
    }

    [webView loadRequest:[NSURLRequest requestWithURL:nsUrl]];

    /* Wait up to 60 s for load to complete. */
    spin_runloop_until(&delegate->_loadDone, 60.0);

    if (!delegate.loadDone) {
        snprintf(wk_err_buf, sizeof(wk_err_buf),
                 "page load timed out after 60 s: %s", url);
        return WK_ERR_LOAD;
    }
    if (delegate.loadFailed) {
        snprintf(wk_err_buf, sizeof(wk_err_buf), "page load failed: %s",
                 delegate.loadError.localizedDescription.UTF8String
                     ?: "unknown error");
        return WK_ERR_LOAD;
    }

    /* JavaScript delay: pump the run loop so timers / async scripts fire. */
    if (js_delay_ms > 0) {
        NSDate *end = [NSDate dateWithTimeIntervalSinceNow:js_delay_ms / 1000.0];
        while ([end timeIntervalSinceNow] > 0) {
            [[NSRunLoop currentRunLoop]
                runMode:NSDefaultRunLoopMode
             beforeDate:[NSDate dateWithTimeIntervalSinceNow:0.05]];
        }
    }

    return WK_OK;
}

/* ═══════════════════════════════════════════════════════════════════════════
 * wk_render_pdf — HTML → PDF via WKWebView + createPDFWithConfiguration
 *
 * Uses the modern (macOS 11+) completion-handler API instead of the legacy
 * NSPrintOperation path, which hangs when run outside a fully-launched
 * Cocoa application.
 * ═══════════════════════════════════════════════════════════════════════════ */

int wk_render_pdf(const char *url, const WkPdfOptions *opts,
                  unsigned char **out_data, size_t *out_len) {
    @autoreleasepool {
        wk_err_buf[0] = '\0';

        if (@available(macOS 11.0, *)) {
            /* all good */
        } else {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "macOS 11.0+ (Big Sur) required for WebKit PDF export");
            return WK_ERR_RENDER;
        }

        /* Ensure NSApplication exists (required for WKWebView off-screen). */
        if (NSApp == nil) {
            [NSApplication sharedApplication];
            [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
        }

        /*
         * createPDFWithConfiguration: paginates content into pages whose
         * dimensions match the WKWebView's frame.  We therefore size the
         * view to the requested *content area* (page minus margins) so
         * that the resulting PDF pages contain properly reflowed text.
         *
         * mm → points: 1 pt = 25.4 / 72 mm.
         */
        double mm2pt = 72.0 / 25.4;
        CGFloat contentW = (opts->page_width_mm
                            - opts->margin_left_mm
                            - opts->margin_right_mm) * mm2pt;
        CGFloat contentH = (opts->page_height_mm
                            - opts->margin_top_mm
                            - opts->margin_bottom_mm) * mm2pt;
        if (contentW < 72) contentW = 72;   /* 1 inch minimum */
        if (contentH < 72) contentH = 72;

        WKWebViewConfiguration *config =
            [[WKWebViewConfiguration alloc] init];
        WKWebView *webView =
            [[WKWebView alloc] initWithFrame:NSMakeRect(0, 0, contentW, contentH)
                               configuration:config];

        /* Hidden window so the compositing layer is active. */
        NSWindow *window = [[NSWindow alloc]
            initWithContentRect:NSMakeRect(0, 0, contentW, contentH)
                      styleMask:NSWindowStyleMaskBorderless
                        backing:NSBackingStoreBuffered
                          defer:NO];
        [window.contentView addSubview:webView];
        webView.frame = window.contentView.bounds;

        WKRenderDelegate *delegate = [[WKRenderDelegate alloc] init];
        webView.navigationDelegate = delegate;

        int rc = load_url(webView, delegate, url, opts->js_delay_ms);
        if (rc != WK_OK) return rc;

        /* ── Generate PDF via the async API ──────────────────────────── */
        __block NSData  *pdfData  = nil;
        __block NSError *pdfError = nil;
        __block BOOL     pdfDone  = NO;

        if (@available(macOS 11.0, *)) {
            WKPDFConfiguration *pdfCfg =
                [[WKPDFConfiguration alloc] init];
            /* Default rect = entire web view, which is exactly what we want. */

            [webView createPDFWithConfiguration:pdfCfg
                              completionHandler:^(NSData *data,
                                                  NSError *error) {
                pdfData  = data;
                pdfError = error;
                pdfDone  = YES;
            }];
        }

        spin_runloop_until(&pdfDone, 60.0);

        if (!pdfDone) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "PDF generation timed out after 60 s");
            return WK_ERR_RENDER;
        }
        if (pdfError || !pdfData) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "PDF generation failed: %s",
                     pdfError.localizedDescription.UTF8String
                         ?: "unknown error");
            return WK_ERR_RENDER;
        }
        if (pdfData.length < 4) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "WebKit produced an empty or invalid PDF");
            return WK_ERR_RENDER;
        }

        *out_len  = pdfData.length;
        *out_data = (unsigned char *)malloc(*out_len);
        if (!*out_data) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "memory allocation failed (%zu bytes)", *out_len);
            return WK_ERR_IO;
        }
        memcpy(*out_data, pdfData.bytes, *out_len);

        return WK_OK;
    }
}

/* ═══════════════════════════════════════════════════════════════════════════
 * wk_render_png — HTML → PNG via WKWebView + takeSnapshot
 * ═══════════════════════════════════════════════════════════════════════════ */

int wk_render_png(const char *url,
                  int viewport_width, int viewport_height,
                  int js_delay_ms,
                  unsigned char **out_data, size_t *out_len) {
    @autoreleasepool {
        wk_err_buf[0] = '\0';

        if (NSApp == nil) {
            [NSApplication sharedApplication];
            [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
        }

        CGFloat w = viewport_width  > 0 ? viewport_width  : 1280;
        CGFloat h = viewport_height > 0 ? viewport_height : 960;

        WKWebViewConfiguration *config =
            [[WKWebViewConfiguration alloc] init];
        WKWebView *webView =
            [[WKWebView alloc] initWithFrame:NSMakeRect(0, 0, w, h)
                               configuration:config];

        NSWindow *window = [[NSWindow alloc]
            initWithContentRect:NSMakeRect(0, 0, w, h)
                      styleMask:NSWindowStyleMaskBorderless
                        backing:NSBackingStoreBuffered
                          defer:NO];
        [window.contentView addSubview:webView];
        webView.frame = window.contentView.bounds;

        WKRenderDelegate *delegate = [[WKRenderDelegate alloc] init];
        webView.navigationDelegate = delegate;

        int rc = load_url(webView, delegate, url, js_delay_ms);
        if (rc != WK_OK) return rc;

        /* Take a snapshot. */
        __block NSImage *snapshot  = nil;
        __block NSError *snapError = nil;
        __block BOOL     snapDone  = NO;

        WKSnapshotConfiguration *snapCfg =
            [[WKSnapshotConfiguration alloc] init];
        snapCfg.rect = webView.bounds;

        [webView takeSnapshotWithConfiguration:snapCfg
                             completionHandler:^(NSImage *image,
                                                 NSError *error) {
            snapshot  = image;
            snapError = error;
            snapDone  = YES;
        }];

        spin_runloop_until(&snapDone, 30.0);

        if (!snapDone) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "snapshot timed out after 30 s");
            return WK_ERR_RENDER;
        }
        if (snapError || !snapshot) {
            snprintf(wk_err_buf, sizeof(wk_err_buf), "snapshot failed: %s",
                     snapError.localizedDescription.UTF8String
                         ?: "unknown error");
            return WK_ERR_RENDER;
        }

        /* Convert NSImage → PNG data. */
        NSData *tiff = [snapshot TIFFRepresentation];
        NSBitmapImageRep *rep =
            [NSBitmapImageRep imageRepWithData:tiff];
        NSData *pngData =
            [rep representationUsingType:NSBitmapImageFileTypePNG
                              properties:@{}];

        if (!pngData || pngData.length == 0) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "failed to encode snapshot as PNG");
            return WK_ERR_RENDER;
        }

        *out_len  = pngData.length;
        *out_data = (unsigned char *)malloc(*out_len);
        if (!*out_data) {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "memory allocation failed (%zu bytes)", *out_len);
            return WK_ERR_IO;
        }
        memcpy(*out_data, pngData.bytes, *out_len);

        return WK_OK;
    }
}
