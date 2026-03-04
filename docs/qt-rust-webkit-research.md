# Qt Rust Wrapper Research: WebKit Compatibility

This document records the research performed for task 2 of the *back-to-roots*
investigation: evaluating whether a Qt Rust wrapper can provide Qt6 + WebKit
rendering as a drop-in replacement for the current headless-Chromium backend.

---

## 1. Candidates Evaluated

Three families of Qt Rust wrappers are available on crates.io:

| Family | Primary crate(s) | Latest version | Approach |
|--------|-----------------|---------------|----------|
| **qmetaobject-rs** | `qmetaobject`, `qttypes` | 0.2.10 / 0.2.12 | Exposes Rust structs to Qt's QMetaObject system; hand-written bindings |
| **ritual-generated bindings** | `qt_core`, `qt_gui`, `qt_widgets`, `qt_qml`, … | 0.5.0 | Automatically generated FFI wrappers produced by the `ritual` tool from Qt5 C++ headers |
| **cxx-qt** | `cxx-qt`, `cxx-qt-lib`, `cxx-qt-build` | 0.8.1 | Safe Rust/C++ interop layer built on top of `cxx`; supports Qt5 and Qt6 |

---

## 2. Qt6 Compatibility

### 2.1 qmetaobject-rs

`qmetaobject` and `qttypes` target **Qt 5** exclusively. The crate README and
source confirm that only Qt5 headers are supported. No Qt6 migration has been
published.

**Qt6 support: ✗ Not available**

### 2.2 ritual-generated bindings

The `ritual` project generated bindings for `qt_core`, `qt_gui`, `qt_widgets`,
`qt_qml`, `qt_3d_core`, `qt_charts`, and `qt_network` against **Qt 5**
headers. The last published versions (0.5.0) are from 2019–2021. The upstream
`ritual` generator project has been unmaintained since 2021 and no Qt6 bindings
have been published.

**Qt6 support: ✗ Not available**

### 2.3 cxx-qt

`cxx-qt` 0.8.x explicitly supports both **Qt 5 and Qt 6** via the
`cxx-qt-build` build-script integration, which detects the installed Qt version
at compile time. The library layer (`cxx-qt-lib`) provides bindings for
`QObject`, `QString`, `QVariant`, `QList`, `QDateTime`, signal/slot connections,
and several other commonly used Qt types against either Qt major version.

**Qt6 support: ✓ Supported (Qt 5 and Qt 6)**

---

## 3. WebKit Module Availability

This is the critical question: can any of these wrappers expose **Qt WebKit**
(specifically `QWebPage`, `QWebFrame`, `QWebPrinter`) to Rust?

### 3.1 The status of Qt WebKit

Qt WebKit was **removed from the mainline Qt repository in Qt 5.6** (2016).
After that release it was moved to a separate community-maintained fork called
*Qt WebKit Reloaded* / *qtwebkit-5.212*, which has seen only sporadic
maintenance and does not support Qt 6.

**Qt 6 ships no WebKit module at all.** The only embedded-browser option in Qt 6
is **Qt WebEngine**, which wraps Chromium (the same engine already used by the
current `headless_chrome` backend).

### 3.2 qt_webkit crate

A search of crates.io for `qt_webkit` returns **no results**. No ritual-generated
or hand-written `qt_webkit` crate has ever been published.

### 3.3 qmetaobject-rs

The `qmetaobject` family contains no WebKit bindings. Its scope is limited to the
QMetaObject/QML layer.

### 3.4 cxx-qt

`cxx-qt-lib` does **not** include any WebKit or WebEngine bindings out of the box.
Custom C++ bridge code could in theory be written with `cxx-qt` to wrap Qt
WebEngine APIs, but this is not a ready-made solution and would require
significant effort to implement from scratch.

---

## 4. Compatibility Matrix

| Wrapper | Qt5 WebKit bindings | Qt6 WebKit bindings | Qt6 WebEngine bindings | Crates.io status |
|---------|--------------------|--------------------|----------------------|-----------------|
| qmetaobject-rs | ✗ | ✗ | ✗ | Published; unmaintained (last: 0.2.10, 2022) |
| ritual (qt_core, qt_widgets, …) | ✗ (no qt_webkit crate) | ✗ | ✗ | Published; unmaintained (last: 0.5.0, 2021) |
| cxx-qt | ✗ (no built-in) | ✗ | ✗ (possible via custom bridge) | Actively maintained (0.8.1, 2024) |

---

## 5. Conclusion and Recommendation

**There is no viable Qt Rust wrapper that provides WebKit bindings for either
Qt5 or Qt6.** The reasons are:

1. **Qt WebKit is deprecated and removed from Qt 5.6+.** No Qt version ships
   WebKit in a supported form today.
2. **Qt6 has no WebKit at all.** Its embedded-browser module is Qt WebEngine
   (Chromium-based).
3. **No `qt_webkit` crate exists on crates.io.** None of the three major wrapper
   families (qmetaobject-rs, ritual, cxx-qt) ship or have planned WebKit
   bindings.
4. **The ritual project is unmaintained** and covers only Qt5; it cannot be used
   to target Qt6.
5. **qmetaobject-rs is Qt5-only and unmaintained.**
6. **cxx-qt 0.8.x** is the only actively-maintained Qt Rust wrapper and supports
   Qt6, but it does not include any browser/WebKit/WebEngine bindings. Writing
   them would require a substantial custom C++ bridge and ongoing maintenance
   work that exceeds the scope of this investigation.

### Recommended path forward

Given that:

* Qt WebKit is abandoned.
* Qt6 + WebKit is not a feasible combination.
* The current `headless_chrome` backend already uses a Chromium engine—which is
  the same engine that Qt WebEngine wraps—the current architecture is already
  functionally equivalent to the "Qt6 + browser" path.

**The existing headless-Chromium approach (`crates/core/src/renderer.rs` via
`headless_chrome = "1"`) is the correct and best-available Rust implementation**
for HTML-to-PDF and HTML-to-image rendering.  Introducing a Qt dependency (via
cxx-qt or any other wrapper) would add native library complexity and OS sandbox
requirements without any rendering-quality improvement over the current backend.

If a pure-Rust, Qt-free WebKit-like alternative is needed in the future, the
closest option is `webkit2gtk` (`webkit2gtk = "2.0.2"` on crates.io), which
provides Rust bindings to the **GTK port of WebKit** (WebKitGTK). That crate is
Linux-only and requires the `libwebkit2gtk` system library; it is not a viable
cross-platform drop-in.

---

## 6. Crate Reference

| Crate | Version on crates.io | Qt target | WebKit? | Active? |
|-------|---------------------|-----------|---------|---------|
| `cxx-qt` | 0.8.1 | Qt 5 + Qt 6 | No | ✓ |
| `cxx-qt-lib` | 0.8.1 | Qt 5 + Qt 6 | No | ✓ |
| `cxx-qt-build` | 0.8.1 | Qt 5 + Qt 6 | No | ✓ |
| `qmetaobject` | 0.2.10 | Qt 5 | No | ✗ |
| `qttypes` | 0.2.12 | Qt 5 | No | ✗ |
| `qt_core` | 0.5.0 | Qt 5 | No | ✗ |
| `qt_gui` | 0.5.0 | Qt 5 | No | ✗ |
| `qt_widgets` | 0.5.0 | Qt 5 | No | ✗ |
| `qt_webkit` | *not published* | — | — | — |
| `webkit2gtk` | 2.0.2 | GTK (not Qt) | Yes (GTK port) | ✓ (Linux only) |
| `headless_chrome` | 1.x | None (CDP) | Via Chromium | ✓ (current) |
