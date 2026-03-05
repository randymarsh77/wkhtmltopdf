# Multi-stage Dockerfile for the wkhtmltopdf GitHub Action.
#
# Stage 1 – builder
#   Compiles the wkhtmltopdf and wkhtmltoimage binaries with the
#   `qt-webkit` feature enabled so that Qt WebEngine is used as the
#   rendering backend.  All Qt6 + WebEngine *development* packages are
#   installed here; they are NOT carried into the runtime image.
#
# Stage 2 – runtime
#   A slim image that contains only the compiled binaries and the Qt6 +
#   WebKit *runtime* shared libraries they depend on.  This keeps the final
#   image small and free of unnecessary build tooling.
#
# Qt6 WebEngine package names (Ubuntu 22.04 "jammy"):
#   Build:   qt6-webengine-dev  (pulls in WebEngineWidgets + WebEngineCore
#             dev headers and static/dynamic libs)
#   Runtime: libqt6webenginewidgets6, libqt6webenginecore6,
#             qt6-webengine-data  (Chromium resources & pak files)
#
# Additional runtime libraries required by Qt6 WebEngine on Linux:
#   libqt6widgets6, libqt6gui6, libqt6network6, libqt6core6,
#   libqt6opengl6, libqt6dbus6 – standard Qt6 modules
#   libnss3, libnspr4 – Chromium NSS crypto (needed by QtWebEngine)
#   libglib2.0-0 – GLib (required by many Chromium components)
#   libxcomposite1, libxdamage1, libxfixes3, libxrandr2 – X11 extensions
#   libfontconfig1, libfreetype6 – font rendering
#   libxshmfence1 – MIT shared memory fence (GPU process)
#   libgbm1 – Mesa GBM (GPU buffer allocation)
#   libegl1 – EGL (OpenGL ES window-system integration)

# ---------------------------------------------------------------------------
# Stage 1: builder
# ---------------------------------------------------------------------------
FROM ubuntu:22.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

# Install Rust via rustup so we pin to the stable channel.
RUN apt-get update && apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"

# Install Qt6 WebEngine *build* dependencies plus generic build tooling.
# qt6-webengine-dev transitively pulls in:
#   qt6-base-dev, libqt6widgets6, libqt6gui6, libqt6network6,
#   libqt6webenginecore6-dev, libqt6webenginewidgets6-dev, cmake, ninja-build
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        pkg-config \
        libssl-dev \
        qt6-webengine-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

# Build only the CLI crate (which pulls in core with qt-webkit enabled).
# The qt-webkit feature activates the cxx-qt bridge to QWebEngineView.
RUN cargo build --release -p wkhtmltopdf-cli --features wkhtmltopdf-core/qt-webkit

# ---------------------------------------------------------------------------
# Stage 2: runtime
# ---------------------------------------------------------------------------
FROM ubuntu:22.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Install Qt6 + WebKit runtime libraries.
#
# Core Qt6 modules:
#   libqt6core6         Qt Core
#   libqt6gui6          Qt GUI (needed by Widgets and WebEngine)
#   libqt6widgets6      Qt Widgets (QApplication, QWebEngineView parent)
#   libqt6network6      Qt Network (HTTP requests inside WebEngine)
#   libqt6opengl6       Qt OpenGL (GPU compositing)
#   libqt6dbus6         Qt D-Bus  (process communication on Linux)
#
# Qt6 WebEngine modules:
#   libqt6webenginecore6      Qt WebEngine Core (Chromium embedding layer)
#   libqt6webenginewidgets6   Qt WebEngine Widgets (QWebEngineView)
#   qt6-webengine-data        Chromium pak/resource files (icudtl.dat, etc.)
#
# System libraries required by Qt6 WebEngine at runtime:
#   libnss3, libnspr4         Chromium NSS TLS/crypto
#   libglib2.0-0              GLib (used throughout Chromium)
#   libxcomposite1            X11 Composite extension
#   libxdamage1               X11 Damage extension
#   libxfixes3                X11 Fixes extension
#   libxrandr2                X11 RandR extension (display configuration)
#   libfontconfig1            Font configuration (text rendering)
#   libfreetype6              FreeType (font rasterisation)
#   libxshmfence1             MIT shared memory fence (GPU sync)
#   libgbm1                   Mesa GBM (GPU buffer management)
#   libegl1                   EGL (OpenGL ES / GPU context creation)
#   ca-certificates           TLS root certificates (HTTPS page loads)
RUN apt-get update && apt-get install -y --no-install-recommends \
        libqt6core6 \
        libqt6gui6 \
        libqt6widgets6 \
        libqt6network6 \
        libqt6opengl6 \
        libqt6dbus6 \
        libqt6webenginecore6 \
        libqt6webenginewidgets6 \
        qt6-webengine-data \
        libnss3 \
        libnspr4 \
        libglib2.0-0 \
        libxcomposite1 \
        libxdamage1 \
        libxfixes3 \
        libxrandr2 \
        libfontconfig1 \
        libfreetype6 \
        libxshmfence1 \
        libgbm1 \
        libegl1 \
        ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binaries from the builder stage.
COPY --from=builder /src/target/release/wkhtmltopdf   /usr/local/bin/wkhtmltopdf
COPY --from=builder /src/target/release/wkhtmltoimage /usr/local/bin/wkhtmltoimage

# The action entrypoint.
COPY action/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
