# wkhtmltopdf v0.13.0 — First Rust-based Release

This is the first release of the **complete Rust reimplementation** of
wkhtmltopdf and wkhtmltoimage.  The Qt/C++ engine has been replaced by a
pure-Rust pipeline backed by headless Chromium.

## Highlights

* **No Qt, no X11** — headless Chromium handles all HTML rendering.  Linux
  users no longer need a display server.
* **Same CLI interface** — existing `wkhtmltopdf` / `wkhtmltoimage` commands
  work without modification.
* **Binary-compatible `libwkhtmltox`** — applications that link the C shared
  library can replace it in-place.
* **PDF/A support** — produce archival-quality PDF/A-1b, A2b, or A3b output.
* **Table of contents** — configurable depth via `--toc-depth`.
* **Document metadata** — set `author`, `subject`, and `title` directly.
* **Full image pipeline** — PNG, JPEG, BMP, SVG output with crop/resize/DPI.

## System requirements

A **Chromium or Google Chrome** binary must be present in `PATH` at runtime.

| Platform | Install |
|---|---|
| Debian / Ubuntu | `sudo apt-get install chromium-browser` |
| macOS | `brew install --cask chromium` |
| Windows | Install [Google Chrome](https://www.google.com/chrome/) |

## Migration from v0.12.x

See [`docs/migration.md`](docs/migration.md) for the full migration guide.

**TL;DR:**

* **CLI users**: no changes required; all existing flags are supported.
* **`libwkhtmltox` users**: replace the shared library in-place; the C ABI is
  fully preserved.  `wkhtmltopdf_version()` now returns `"0.13.0"`.
* **Rendering**: pages may look slightly different because Chromium has
  broader CSS support than Qt WebKit — this is generally an improvement.
* **Breaking**: Chromium must be installed at runtime (Qt is no longer
  bundled).

## What's new

* New `--toc-depth N` flag (default: 3).
* New `--pdf-a` flag for PDF/A output.
* New `--author` / `--subject` flags for document metadata.
* New `--dpi N` flag for `wkhtmltoimage`.
* Header/footer variables: `[page]`, `[toPage]`, `[date]`, `[title]`, `[url]`.
* Rust library crates published: `wkhtmltopdf-settings`, `wkhtmltopdf-core`,
  `wkhtmltopdf-pdf`, `wkhtmltopdf-image`.

## Breaking changes

* Chromium required at runtime (no longer bundled).
* `--disable-javascript` / `--enable-javascript` accepted but not enforced
  (Chromium always runs JavaScript).
* Qt-specific CSS extensions removed.
* PostScript output not supported (removed in v0.12.1).

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md) for the full history.
