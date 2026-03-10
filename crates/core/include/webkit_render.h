/*
 * webkit_render.h — Cross-platform headless WebKit rendering helpers.
 *
 * macOS:  Implemented via WKWebView (WebKit.framework + AppKit.framework).
 * Linux:  Implemented via WebKitGTK (libwebkit2gtk + GTK 3).
 *
 * These functions are called from Rust through FFI defined in
 * crates/core/src/webkit.rs.  They are synchronous: each call blocks the
 * calling thread until the render completes or times out.
 */

#ifndef WEBKIT_RENDER_H
#define WEBKIT_RENDER_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Return codes ────────────────────────────────────────────────────────── */

#define WK_OK            0
#define WK_ERR_INIT     -1   /* toolkit / display initialisation failed    */
#define WK_ERR_LOAD     -2   /* page load failed or timed out              */
#define WK_ERR_RENDER   -3   /* PDF / screenshot creation failed           */
#define WK_ERR_IO       -4   /* I/O error (temp file, memory allocation)   */

/* ── PDF rendering options ───────────────────────────────────────────────── */

typedef struct {
    double page_width_mm;
    double page_height_mm;
    double margin_top_mm;
    double margin_bottom_mm;
    double margin_left_mm;
    double margin_right_mm;
    int    print_backgrounds;
    int    js_delay_ms;
} WkPdfOptions;

/* ── Functions ───────────────────────────────────────────────────────────── */

/**
 * Render an HTML page to PDF via WebKit.
 *
 * @param url        URL string: http://, https://, file://, or a bare
 *                   local path (which is converted to file:// internally).
 * @param opts       PDF rendering options (page size, margins, etc.).
 * @param out_data   On success, receives a malloc'd buffer with the PDF
 *                   bytes.  The caller must free it with wk_free().
 * @param out_len    On success, receives the length of *out_data.
 * @return           WK_OK on success; a negative WK_ERR_* code on failure.
 *                   Call wk_last_error() for a human-readable message.
 */
int wk_render_pdf(const char *url,
                  const WkPdfOptions *opts,
                  unsigned char **out_data,
                  size_t *out_len);

/**
 * Render an HTML page to a PNG screenshot via WebKit.
 *
 * @param url              URL string (same formats as wk_render_pdf).
 * @param viewport_width   Viewport width in pixels (0 = default 1280).
 * @param viewport_height  Viewport height in pixels (0 = default 960).
 * @param js_delay_ms      Milliseconds to wait after page load.
 * @param out_data         On success, receives malloc'd PNG bytes.
 * @param out_len          On success, receives the length of *out_data.
 * @return                 WK_OK on success; negative WK_ERR_* on failure.
 */
int wk_render_png(const char *url,
                  int viewport_width,
                  int viewport_height,
                  int js_delay_ms,
                  unsigned char **out_data,
                  size_t *out_len);

/**
 * Return a human-readable error message from the last failed call.
 * The returned pointer is thread-local and valid until the next call
 * from the same thread.
 */
const char *wk_last_error(void);

/** Free memory allocated by wk_render_pdf / wk_render_png. */
void wk_free(void *ptr);

#ifdef __cplusplus
}
#endif

#endif /* WEBKIT_RENDER_H */
