---
layout: default
---

# Ubuntu 25.04 (Plucky Piculet) Build Notes

Ubuntu 25.04 ("Plucky Piculet") no longer ships the legacy `libqt4-webkit` or
`libqt5webkit5-dev` packages.  The Rust-based rewrite of wkhtmltopdf uses a
headless Chromium browser for rendering, so the dependency set is different from
earlier Ubuntu releases.

## System Dependencies

Install the following packages before building or running wkhtmltopdf on
Ubuntu 25.04:

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    chromium-browser \
    ca-certificates
```

| Package | Purpose |
|---------|---------|
| `build-essential` | C/C++ compiler and linker toolchain required by some Rust crate build scripts |
| `pkg-config` | Locates native libraries for Rust build scripts |
| `libssl-dev` | OpenSSL development headers (required by the `openssl` crate used for HTTPS fetching) |
| `chromium-browser` | Headless Chromium browser used by the rendering backend |
| `ca-certificates` | TLS CA bundle required for HTTPS URL fetching |

## Installing Rust

wkhtmltopdf is now a Rust project.  Install the Rust toolchain with:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

The minimum supported Rust version (MSRV) is **1.80**.

## Building from Source

```bash
git clone https://github.com/wkhtmltopdf/wkhtmltopdf.git
cd wkhtmltopdf
cargo build --release
```

The compiled binaries will be placed in `target/release/`:

* `target/release/wkhtmltopdf`
* `target/release/wkhtmltoimage`

## Verifying Chromium Headless Availability

The rendering backend launches Chromium in headless mode.  Verify that it is
accessible before running wkhtmltopdf:

```bash
chromium-browser --headless --disable-gpu --dump-dom about:blank
```

If Chromium is installed under a different name (e.g. `chromium`), create a
symlink or set the `CHROME_PATH` environment variable:

```bash
export CHROME_PATH=$(which chromium)
```

## Notes on Qt/WebKit Availability

Ubuntu 25.04 does **not** provide `libqt5webkit5-dev` or any equivalent
Qt WebKit package.  The legacy C++/Qt build path (`wkhtmltopdf.pro`,
`common.pri`) is therefore **not** supported on this release.  All Ubuntu 25.04
users should use the Rust implementation which ships pre-built binaries and
supports headless Chromium as the rendering engine.
