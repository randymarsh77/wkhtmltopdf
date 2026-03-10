/*
 * webkit_render_linux.c — WebKit rendering helpers for Linux.
 *
 * Uses WebKitGTK (webkit2gtk) for both PDF and PNG output.
 * Compiled by the cc crate when the target is Linux and
 * pkg-config finds webkit2gtk-4.1 (or webkit2gtk-4.0).
 *
 * PDF:  WebKitPrintOperation with "Print to File" settings.
 * PNG:  webkit_web_view_get_snapshot → cairo → PNG bytes.
 */

#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#include <cairo.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>
#include "webkit_render.h"

/* ── Thread-local error buffer ───────────────────────────────────────────── */

static _Thread_local char wk_err_buf[2048] = {0};

const char *wk_last_error(void) {
    return wk_err_buf;
}

void wk_free(void *ptr) {
    free(ptr);
}

/* ── GTK initialization ──────────────────────────────────────────────────── */

static gboolean gtk_inited = FALSE;

static int ensure_gtk(void) {
    if (gtk_inited) return 0;
    gtk_inited = gtk_init_check(NULL, NULL);
    if (!gtk_inited) {
        snprintf(wk_err_buf, sizeof(wk_err_buf),
                 "GTK init failed – no display available? "
                 "Try: GDK_BACKEND=x11 xvfb-run <command>");
        return WK_ERR_INIT;
    }
    return WK_OK;
}

/* ── Load context (shared between load callbacks and main loop) ──────── */

typedef struct {
    GMainLoop *loop;
    gboolean   done;
    gboolean   failed;
    char       msg[1024];
} LoadCtx;

static void on_load_changed(WebKitWebView *wv, WebKitLoadEvent event,
                             gpointer data) {
    LoadCtx *ctx = data;
    if (event == WEBKIT_LOAD_FINISHED) {
        ctx->done = TRUE;
        g_main_loop_quit(ctx->loop);
    }
}

static gboolean on_load_failed(WebKitWebView *wv, WebKitLoadEvent event,
                                const gchar *uri, GError *err,
                                gpointer data) {
    LoadCtx *ctx = data;
    snprintf(ctx->msg, sizeof(ctx->msg), "%s", err->message);
    ctx->failed = TRUE;
    ctx->done   = TRUE;
    g_main_loop_quit(ctx->loop);
    return TRUE;   /* handled */
}

static gboolean load_timeout(gpointer data) {
    LoadCtx *ctx = data;
    snprintf(ctx->msg, sizeof(ctx->msg), "page load timed out (60 s)");
    ctx->failed = TRUE;
    ctx->done   = TRUE;
    g_main_loop_quit(ctx->loop);
    return G_SOURCE_REMOVE;
}

/* ── JS-delay helper ─────────────────────────────────────────────────────── */

static gboolean js_delay_done(gpointer data) {
    g_main_loop_quit((GMainLoop *)data);
    return G_SOURCE_REMOVE;
}

/* ── Common: create web view, load URL, wait ─────────────────────────────── */

static int load_page(const char *url, int js_delay_ms,
                     WebKitWebView **out_wv, GMainLoop **out_loop) {
    int rc = ensure_gtk();
    if (rc != WK_OK) return rc;

    GMainLoop *loop = g_main_loop_new(NULL, FALSE);

    WebKitWebView *wv = WEBKIT_WEB_VIEW(webkit_web_view_new());
    g_object_ref_sink(wv);

    LoadCtx ctx = { .loop = loop, .done = FALSE, .failed = FALSE };
    ctx.msg[0] = '\0';

    g_signal_connect(wv, "load-changed",
                     G_CALLBACK(on_load_changed), &ctx);
    g_signal_connect(wv, "load-failed",
                     G_CALLBACK(on_load_failed), &ctx);

    guint timer = g_timeout_add_seconds(60, load_timeout, &ctx);

    /* Normalise bare paths to file:// URLs. */
    if (strncmp(url, "http://", 7) != 0 &&
        strncmp(url, "https://", 8) != 0 &&
        strncmp(url, "file://", 7) != 0) {
        char abs[PATH_MAX];
        if (realpath(url, abs)) {
            char file_url[PATH_MAX + 8];
            snprintf(file_url, sizeof(file_url), "file://%s", abs);
            webkit_web_view_load_uri(wv, file_url);
        } else {
            char file_url[PATH_MAX + 8];
            snprintf(file_url, sizeof(file_url), "file://%s", url);
            webkit_web_view_load_uri(wv, file_url);
        }
    } else {
        webkit_web_view_load_uri(wv, url);
    }

    g_main_loop_run(loop);
    g_source_remove(timer);

    if (ctx.failed) {
        snprintf(wk_err_buf, sizeof(wk_err_buf), "%s", ctx.msg);
        g_object_unref(wv);
        g_main_loop_unref(loop);
        return WK_ERR_LOAD;
    }

    /* JS delay. */
    if (js_delay_ms > 0) {
        g_timeout_add((guint)js_delay_ms, js_delay_done, loop);
        g_main_loop_run(loop);
    }

    *out_wv   = wv;
    *out_loop = loop;
    return WK_OK;
}

/* ═══════════════════════════════════════════════════════════════════════════
 * wk_render_pdf — HTML → PDF via WebKitPrintOperation
 * ═══════════════════════════════════════════════════════════════════════════ */

typedef struct {
    GMainLoop *loop;
    gboolean   done;
    gboolean   ok;
    char       msg[1024];
} PrintCtx;

static void on_print_finished(WebKitPrintOperation *op, gpointer data) {
    PrintCtx *ctx = data;
    ctx->ok   = TRUE;
    ctx->done = TRUE;
    g_main_loop_quit(ctx->loop);
}

static void on_print_failed(WebKitPrintOperation *op, GError *err,
                             gpointer data) {
    PrintCtx *ctx = data;
    snprintf(ctx->msg, sizeof(ctx->msg), "%s",
             err ? err->message : "unknown print error");
    ctx->ok   = FALSE;
    ctx->done = TRUE;
    g_main_loop_quit(ctx->loop);
}

static gboolean print_timeout(gpointer data) {
    PrintCtx *ctx = data;
    snprintf(ctx->msg, sizeof(ctx->msg), "print operation timed out (120 s)");
    ctx->ok   = FALSE;
    ctx->done = TRUE;
    g_main_loop_quit(ctx->loop);
    return G_SOURCE_REMOVE;
}

int wk_render_pdf(const char *url, const WkPdfOptions *opts,
                  unsigned char **out_data, size_t *out_len) {
    wk_err_buf[0] = '\0';

    WebKitWebView *wv   = NULL;
    GMainLoop     *loop = NULL;
    int rc = load_page(url, opts->js_delay_ms, &wv, &loop);
    if (rc != WK_OK) return rc;

    /* Create temp file path for the PDF output. */
    char tmp_path[] = "/tmp/wk_pdf_XXXXXX";
    int fd = mkstemp(tmp_path);
    if (fd < 0) {
        snprintf(wk_err_buf, sizeof(wk_err_buf), "mkstemp failed");
        g_object_unref(wv);
        g_main_loop_unref(loop);
        return WK_ERR_IO;
    }
    close(fd);

    char output_uri[PATH_MAX + 8];
    snprintf(output_uri, sizeof(output_uri), "file://%s", tmp_path);

    /* ── Configure print settings ────────────────────────────────────── */
    GtkPrintSettings *ps = gtk_print_settings_new();
    gtk_print_settings_set_printer(ps, "Print to File");
    gtk_print_settings_set(ps, GTK_PRINT_SETTINGS_OUTPUT_FILE_FORMAT, "pdf");
    gtk_print_settings_set(ps, GTK_PRINT_SETTINGS_OUTPUT_URI, output_uri);

    /* Page setup. */
    GtkPageSetup *page_setup = gtk_page_setup_new();
    GtkPaperSize *paper = gtk_paper_size_new_custom(
        "custom", "Custom",
        opts->page_width_mm, opts->page_height_mm, GTK_UNIT_MM);
    gtk_page_setup_set_paper_size(page_setup, paper);
    gtk_page_setup_set_top_margin(page_setup,
                                  opts->margin_top_mm,    GTK_UNIT_MM);
    gtk_page_setup_set_bottom_margin(page_setup,
                                     opts->margin_bottom_mm, GTK_UNIT_MM);
    gtk_page_setup_set_left_margin(page_setup,
                                   opts->margin_left_mm,   GTK_UNIT_MM);
    gtk_page_setup_set_right_margin(page_setup,
                                    opts->margin_right_mm,  GTK_UNIT_MM);

    /* ── Print operation ─────────────────────────────────────────────── */
    WebKitPrintOperation *print_op = webkit_print_operation_new(wv);
    webkit_print_operation_set_print_settings(print_op, ps);
    webkit_print_operation_set_page_setup(print_op, page_setup);

    PrintCtx pctx = { .loop = loop, .done = FALSE, .ok = FALSE };
    pctx.msg[0] = '\0';

    g_signal_connect(print_op, "finished",
                     G_CALLBACK(on_print_finished), &pctx);
    g_signal_connect(print_op, "failed",
                     G_CALLBACK(on_print_failed), &pctx);

    guint ptimer = g_timeout_add_seconds(120, print_timeout, &pctx);

    /* Print headlessly (no dialog). */
    webkit_print_operation_print(print_op);
    g_main_loop_run(loop);
    g_source_remove(ptimer);

    /* ── Read the output file ────────────────────────────────────────── */
    int result = WK_OK;
    if (pctx.ok) {
        FILE *f = fopen(tmp_path, "rb");
        if (f) {
            fseek(f, 0, SEEK_END);
            long sz = ftell(f);
            fseek(f, 0, SEEK_SET);
            if (sz > 0) {
                *out_data = (unsigned char *)malloc((size_t)sz);
                if (*out_data) {
                    *out_len = fread(*out_data, 1, (size_t)sz, f);
                } else {
                    snprintf(wk_err_buf, sizeof(wk_err_buf),
                             "malloc failed (%ld bytes)", sz);
                    result = WK_ERR_IO;
                }
            } else {
                snprintf(wk_err_buf, sizeof(wk_err_buf),
                         "WebKit produced an empty PDF");
                result = WK_ERR_RENDER;
            }
            fclose(f);
        } else {
            snprintf(wk_err_buf, sizeof(wk_err_buf),
                     "could not open PDF temp file: %s", tmp_path);
            result = WK_ERR_IO;
        }
    } else {
        snprintf(wk_err_buf, sizeof(wk_err_buf),
                 "print failed: %s", pctx.msg);
        result = WK_ERR_RENDER;
    }

    /* ── Cleanup ─────────────────────────────────────────────────────── */
    unlink(tmp_path);
    g_object_unref(print_op);
    gtk_paper_size_free(paper);
    g_object_unref(page_setup);
    g_object_unref(ps);
    g_object_unref(wv);
    g_main_loop_unref(loop);

    return result;
}

/* ═══════════════════════════════════════════════════════════════════════════
 * wk_render_png — HTML → PNG via WebKit snapshot API
 * ═══════════════════════════════════════════════════════════════════════════ */

typedef struct {
    GMainLoop       *loop;
    gboolean         done;
    cairo_surface_t *surface;
    char             msg[1024];
} SnapCtx;

static void on_snapshot_ready(GObject *src, GAsyncResult *res, gpointer data) {
    SnapCtx *ctx = data;
    GError  *err = NULL;
    ctx->surface =
        webkit_web_view_get_snapshot_finish(WEBKIT_WEB_VIEW(src), res, &err);
    if (err) {
        snprintf(ctx->msg, sizeof(ctx->msg), "%s", err->message);
        g_error_free(err);
    }
    ctx->done = TRUE;
    g_main_loop_quit(ctx->loop);
}

static gboolean snap_timeout(gpointer data) {
    SnapCtx *ctx = data;
    snprintf(ctx->msg, sizeof(ctx->msg), "snapshot timed out (30 s)");
    ctx->done = TRUE;
    g_main_loop_quit(ctx->loop);
    return G_SOURCE_REMOVE;
}

/* Cairo-write-to-buffer callback. */
typedef struct {
    unsigned char *buf;
    size_t         len;
    size_t         cap;
} PngBuf;

static cairo_status_t png_write_cb(void *closure, const unsigned char *data,
                                   unsigned int length) {
    PngBuf *pb = closure;
    size_t needed = pb->len + length;
    if (needed > pb->cap) {
        size_t newcap = pb->cap ? pb->cap * 2 : 65536;
        if (newcap < needed) newcap = needed;
        unsigned char *tmp = realloc(pb->buf, newcap);
        if (!tmp) return CAIRO_STATUS_NO_MEMORY;
        pb->buf = tmp;
        pb->cap = newcap;
    }
    memcpy(pb->buf + pb->len, data, length);
    pb->len += length;
    return CAIRO_STATUS_SUCCESS;
}

int wk_render_png(const char *url,
                  int viewport_width, int viewport_height,
                  int js_delay_ms,
                  unsigned char **out_data, size_t *out_len) {
    wk_err_buf[0] = '\0';

    /* Viewport sizing: WebKitGTK sizes via the widget allocation;
     * here we just set up the web view (not in a window) and rely on
     * the snapshot to capture the full document. */
    (void)viewport_width;
    (void)viewport_height;

    WebKitWebView *wv   = NULL;
    GMainLoop     *loop = NULL;
    int rc = load_page(url, js_delay_ms, &wv, &loop);
    if (rc != WK_OK) return rc;

    /* Take a snapshot. */
    SnapCtx sctx = { .loop = loop, .done = FALSE, .surface = NULL };
    sctx.msg[0] = '\0';

    guint stimer = g_timeout_add_seconds(30, snap_timeout, &sctx);

    webkit_web_view_get_snapshot(
        wv,
        WEBKIT_SNAPSHOT_REGION_FULL_DOCUMENT,
        WEBKIT_SNAPSHOT_OPTIONS_NONE,
        NULL,               /* cancellable */
        on_snapshot_ready,
        &sctx);

    g_main_loop_run(loop);
    g_source_remove(stimer);

    if (!sctx.surface) {
        snprintf(wk_err_buf, sizeof(wk_err_buf),
                 "snapshot failed: %s",
                 sctx.msg[0] ? sctx.msg : "unknown error");
        g_object_unref(wv);
        g_main_loop_unref(loop);
        return WK_ERR_RENDER;
    }

    /* Encode cairo_surface_t → PNG in memory. */
    PngBuf pb = { .buf = NULL, .len = 0, .cap = 0 };
    cairo_status_t cs =
        cairo_surface_write_to_png_stream(sctx.surface, png_write_cb, &pb);
    cairo_surface_destroy(sctx.surface);

    g_object_unref(wv);
    g_main_loop_unref(loop);

    if (cs != CAIRO_STATUS_SUCCESS || !pb.buf) {
        free(pb.buf);
        snprintf(wk_err_buf, sizeof(wk_err_buf),
                 "PNG encoding failed: %s", cairo_status_to_string(cs));
        return WK_ERR_RENDER;
    }

    *out_data = pb.buf;
    *out_len  = pb.len;
    return WK_OK;
}
