# wkhtmltopdf Codebase Architecture

This document catalogs the major subsystems of the wkhtmltopdf C++/Qt codebase, their interfaces, and their dependencies.

## Repository Layout

```
src/
  lib/      # Core library (libwkhtmltox) – shared by both tools
  shared/   # Shared CLI infrastructure
  pdf/      # wkhtmltopdf command-line driver
  image/    # wkhtmltoimage command-line driver
docs/       # Project documentation and generated API docs
qt/         # Patched Qt (when building with the Qt hack)
```

---

## Major Subsystems

### 1. PDF Rendering (`src/lib/pdfconverter.*`)

**Purpose:** Orchestrates the full HTML-to-PDF pipeline.

**Key files:**

| File | Role |
|------|------|
| `pdfconverter.hh` | Public API (`PdfConverter`) |
| `pdfconverter_p.hh` | Private implementation (`PdfConverterPrivate`, `PageObject`) |
| `pdfconverter.cc` | Conversion logic |

**Public interface – `PdfConverter` (inherits `Converter`):**

```cpp
class PdfConverter : public Converter {
public:
    PdfConverter(settings::PdfGlobal & globalSettings);
    void addResource(const settings::PdfObject & pageSettings,
                     const QString * data = nullptr);
    int pageCount();
    const QByteArray & output();
signals:
    void producingForms(bool);
};
```

**Conversion pipeline (slot sequence in `PdfConverterPrivate`):**

1. `beginConvert()` – initialise printer, set up page objects
2. `pagesLoaded(bool)` – triggered when `MultiPageLoader` finishes; starts TOC/HF loading
3. `measuringHeadersLoaded(bool)` – measure header/footer heights
4. `tocLoaded(bool)` – TOC HTML rendered; may iterate until stable page count
5. `headersLoaded(bool)` – header/footer pages ready
6. `printDocument()` – spool all pages to `QPrinter`

**Key internal class – `PageObject`:**

| Member | Type | Purpose |
|--------|------|---------|
| `settings` | `PdfObject` | Per-page configuration |
| `loaderObject` | `LoaderObject *` | Loaded web page wrapper |
| `page` | `QWebPage *` | Rendered Qt web page |
| `anchors` | `QHash<QString, QWebElement>` | Anchor elements for link resolution |
| `localLinks` / `externalLinks` | `QVector<QPair<QWebElement,QString>>` | Hyperlinks to resolve |
| `headers` / `footers` | `QList<QWebPage *>` | Rendered header/footer pages |
| `pageCount` | `int` | Number of PDF pages this object spans |
| `tocFile` | `TempFile` | Temporary TOC XSLT output file |

**Dependencies:** `Converter`, `MultiPageLoader`, `Outline`, `PdfGlobal`/`PdfObject`, `QPrinter`/`QPainter`, `TempFile`

---

### 2. Image Rendering (`src/lib/imageconverter.*`)

**Purpose:** Converts a single HTML page to a raster image (PNG, JPEG, etc.).

**Key files:**

| File | Role |
|------|------|
| `imageconverter.hh` | Public API (`ImageConverter`) |
| `imageconverter_p.hh` | Private implementation (`ImageConverterPrivate`) |
| `imageconverter.cc` | Conversion logic |

**Public interface – `ImageConverter` (inherits `Converter`):**

```cpp
class ImageConverter : public Converter {
public:
    ImageConverter(settings::ImageGlobal & settings,
                   const QString * data = nullptr);
    const QByteArray & output();
};
```

**Dependencies:** `Converter`, `MultiPageLoader`, `ImageGlobal`

---

### 3. Base Converter (`src/lib/converter.*`)

**Purpose:** Abstract base class shared by both PDF and image converters. Provides the public Qt signal/slot interface and progress reporting.

**Key files:**

| File | Role |
|------|------|
| `converter.hh` | Public API (`Converter`) |
| `converter_p.hh` | Private base (`ConverterPrivate`) |
| `converter.cc` | Common implementation |

**Public interface – `Converter`:**

```cpp
class Converter : public QObject {
public:
    int currentPhase();
    int phaseCount();
    QString phaseDescription(int phase = -1);
    QString progressString();
    int httpErrorCode();
public slots:
    void beginConversion();
    bool convert();
    void cancel();
signals:
    void debug(const QString & message);
    void info(const QString & message);
    void warning(const QString & message);
    void error(const QString & message);
    void phaseChanged();
    void progressChanged(int progress);
    void finished(bool ok);
    // SVG overrides for form elements
    void checkboxSvgChanged(const QString & path);
    void checkboxCheckedSvgChanged(const QString & path);
    void radiobuttonSvgChanged(const QString & path);
    void radiobuttonCheckedSvgChanged(const QString & path);
};
```

---

### 4. Network & Page Loading (`src/lib/multipageloader.*`)

**Purpose:** Loads one or more HTML pages via QtWebKit, handling all network concerns (proxies, SSL, authentication, cookies, custom headers, local-file access restrictions).

**Key files:**

| File | Role |
|------|------|
| `multipageloader.hh` | Public API (`MultiPageLoader`, `LoaderObject`) |
| `multipageloader_p.hh` | Private implementation |
| `multipageloader.cc` | Full implementation |

**Public interface – `MultiPageLoader`:**

```cpp
class MultiPageLoader : public QObject {
public:
    MultiPageLoader(settings::LoadGlobal & s, int dpi,
                    bool mainLoader = false);
    LoaderObject * addResource(const QString & url,
                               const settings::LoadPage & settings,
                               const QString * data = nullptr);
    LoaderObject * addResource(const QUrl & url,
                               const settings::LoadPage & settings);
    static QUrl guessUrlFromString(const QString & string);
    int httpErrorCode();
public slots:
    void load();
    void clearResources();
    void cancel();
signals:
    void loadFinished(bool ok);
    void loadProgress(int progress);
    void loadStarted();
    void debug(QString text);
    void info(QString text);
    void warning(QString text);
    void error(QString text);
};
```

**Internal components (in `multipageloader.cc`):**

| Class | Purpose |
|-------|---------|
| `MyNetworkAccessManager` | Custom `QNetworkAccessManager`; enforces local-file blocking, injects custom headers, manages SSL client certs |
| `MyNetworkProxyFactory` | Routes requests through the configured proxy; supports per-host bypass |
| `MyQWebPage` | `QWebPage` subclass; handles JS dialogs, `window.status` polling, `runScript` execution, and form-field SVG replacement |
| `ResourceObject` | Wraps a single `QWebPage` load lifecycle; manages cookies, auth prompts, JS delay timer, and load-error handling |

**Dependencies:** `LoadGlobal`, `LoadPage`, `QWebPage`, `QNetworkAccessManager`, `QNetworkProxyFactory`

---

### 5. Settings (`src/lib/*settings.*`)

Settings are plain structs populated by the CLI parsers or the C API and then passed into the converters and loaders.

#### 5a. PDF Settings (`pdfsettings.hh`)

| Struct | Purpose |
|--------|---------|
| `Margin` | Page margins (top, right, bottom, left) as `UnitReal` pairs |
| `Size` | Page size (`QPrinter::PageSize`, explicit width/height) |
| `TableOfContent` | TOC behaviour (dotted lines, caption, indentation, font scaling, forward/back links) |
| `HeaderFooter` | Text header/footer (font, left/center/right text, separator line, HTML URL, spacing) |
| `PdfGlobal` | Global PDF options (size, margins, orientation, color mode, DPI, compression, outline depth, output path) |
| `PdfObject` | Per-page PDF options (source URL, TOC, header, footer, link handling, form production, replacement pairs) |

Helper conversion functions (string ↔ enum): `strToPageSize`, `strToOrientation`, `strToColorMode`, `strToUnitReal`, etc.

#### 5b. Load Settings (`loadsettings.hh`)

| Struct | Purpose |
|--------|---------|
| `Proxy` | Proxy type, host, port, credentials |
| `PostItem` | HTTP POST field (name, value, is-file flag) |
| `LoadGlobal` | Global load options (cookie jar path) |
| `LoadPage` | Per-page network options: HTTP auth, SSL client certs, JS delay, `windowStatus`, zoom, custom headers, cookies, POST data, local-file access control, proxy, `runScript` list, SVG form overrides, cache directory, media type |

`LoadPage::LoadErrorHandling` enum: `abort`, `skip`, `ignore`

#### 5c. Web/Rendering Settings (`websettings.hh`)

| Struct | Purpose |
|--------|---------|
| `Web` | Background images, image loading, JavaScript enable, intelligent shrinking, minimum font size, default encoding, user stylesheet, plugins |

#### 5d. Image Settings (`imagesettings.hh`)

| Struct | Purpose |
|--------|---------|
| `CropSettings` | Crop rectangle (left, top, width, height) |
| `ImageGlobal` | All image options: crop, load/web settings, log level, transparency, screen dimensions, quality, output format/path |

---

### 6. CLI Argument Parsing (`src/shared/` and `src/pdf/`, `src/image/`)

**Purpose:** Parses command-line arguments and populates settings structs.

**Key files:**

| File | Role |
|------|------|
| `src/shared/commandlineparserbase.hh/.cc` | Abstract base: argument registration, help/man/html doc output, stdin reading |
| `src/shared/arghandler.*` | Argument handler types |
| `src/shared/outputter.hh` | Pluggable output formatter interface |
| `src/pdf/pdfcommandlineparser.hh/.cc` | PDF-specific parser; maps CLI flags to `PdfGlobal`/`PdfObject` |
| `src/pdf/pdfarguments.cc` | PDF argument definitions |
| `src/pdf/pdfdocparts.cc` | PDF manual sections |
| `src/image/imagecommandlineparser.hh/.cc` | Image-specific parser; maps CLI flags to `ImageGlobal` |

**Flow:**

```
argv
  └─→ PdfCommandLineParser::parse()
        ├─→ PdfGlobal (global options)
        └─→ PdfObject (per-page options, one per page/toc argument)
```

---

### 7. TOC / Outline Generation (`src/lib/outline.*`)

**Purpose:** Builds the document outline (bookmarks) and generates the Table of Contents page.

Requires the `__EXTENSIVE_WKHTMLTOPDF_QT_HACK__` build flag.

**Key files:**

| File | Role |
|------|------|
| `outline.hh` | Public API (`Outline`) |
| `outline_p.hh` | Private implementation (`OutlinePrivate`, `TocPrinter`) |
| `outline.cc` | Tree construction and TOC rendering |
| `tocstylesheet.cc` | Default XSLT stylesheet for TOC HTML generation |

**Public interface – `Outline`:**

```cpp
class Outline {
public:
    Outline(const settings::PdfGlobal & settings);
    void addEmptyWebPage();
    void addWebPage(const QString & name, QWebPrinter & wp,
                    QWebFrame * frame, const settings::PdfObject & ps,
                    QVector<QPair<QWebElement, QString>> & local,
                    QHash<QString, QWebElement> & external);
    bool replaceWebPage(int d, const QString & name, QWebPrinter & wp,
                        QWebFrame * f, const settings::PdfObject & ps,
                        QVector<QPair<QWebElement, QString>> & local,
                        QHash<QString, QWebElement> & anchors);
    void fillHeaderFooterParms(int page,
                               QHash<QString, QString> & parms,
                               const settings::PdfObject & ps);
    void fillAnchors(int d, QHash<QString, QWebElement> & anchors);
    int pageCount();
    void printOutline(QPrinter * printer);
    void dump(QTextStream & stream) const;
};
```

**Dependencies:** `PdfGlobal`, `PdfObject`, `QWebFrame`, `QWebElement`, `QWebPrinter` (patched Qt), `QPrinter`

---

### 8. JavaScript Evaluation

JavaScript evaluation is embedded inside the network/page-loading subsystem (`MyQWebPage` / `ResourceObject` in `multipageloader.cc`) rather than being a standalone subsystem.

**Mechanism:**

1. `QWebPage` renders the page; Qt's built-in JavaScriptCore engine runs inline JS.
2. After the initial `loadFinished` signal, `ResourceObject` waits an additional `jsdelay` milliseconds.
3. If `windowStatus` is set, the loader polls `window.status` every 200 ms until the value matches or the timeout expires.
4. Each script in `LoadPage::runScript` is executed via `QWebFrame::evaluateJavaScript()`.
5. `LoadPage::stopSlowScripts` enables a watchdog that kills scripts exceeding a time limit.
6. `LoadPage::debugJavascript` routes `console.log` / `javaScriptConsoleMessage` output to the converter's `debug` signal.

**Configuration surface (all in `LoadPage`):**

| Setting | Effect |
|---------|--------|
| `enableJavascript` (in `Web`) | Master JS on/off switch |
| `jsdelay` | Extra milliseconds to wait after page load |
| `windowStatus` | Poll `window.status` until this value is seen |
| `runScript` | List of JS snippets to inject after load |
| `stopSlowScripts` | Kill scripts that run too long |
| `debugJavascript` | Forward JS console output to debug log |

---

### 9. Headers & Footers

Headers and footers are managed inside `PdfConverterPrivate` and depend on both the `Outline` and `MultiPageLoader` subsystems.

**Rendering flow:**

1. For each `PageObject` that has a header or footer URL, `PdfConverterPrivate::loadHeaders()` adds the URL to `hfLoader` (a separate `MultiPageLoader`).
2. Header/footer placeholder variables (e.g. `[page]`, `[topage]`, `[section]`) are filled via `fillParms()` and `Outline::fillHeaderFooterParms()`.
3. The URL is rendered with substituted values by `loadHeaderFooter()`, which returns a `QWebPage *`.
4. `calculateHeaderHeight()` measures the rendered height so the main page can reserve the correct amount of margin space.
5. During `spoolPage()`, `handleHeader()` / `handleFooter()` paint the header/footer `QWebPage` onto the `QPainter` at the top/bottom of each physical page.

**Available placeholder variables in header/footer text and URLs:**

| Variable | Value |
|----------|-------|
| `[page]` | Current page number (with offset applied) |
| `[topage]` | Total page count |
| `[section]` | Current section heading |
| `[subsection]` | Current sub-section heading |
| `[doctitle]` | Document title |
| `[isodate]` | Current date (ISO 8601) |
| `[date]` | Current date (locale format) |
| `[time]` | Current time |

---

## Dependency Graph

```
CLI Parsers (PdfCommandLineParser / ImageCommandLineParser)
  │
  │  populate
  ▼
Settings (PdfGlobal / PdfObject / ImageGlobal / LoadGlobal / LoadPage / Web)
  │
  │  passed to
  ▼
PdfConverter / ImageConverter  (both extend Converter)
  │
  ├──── MultiPageLoader  ◄─── LoadGlobal / LoadPage
  │         │
  │         ├── MyNetworkAccessManager  ◄── proxy, SSL, custom headers
  │         ├── MyNetworkProxyFactory
  │         ├── MyQWebPage              ◄── JS evaluation, runScript
  │         └── ResourceObject         ◄── cookies, auth, load errors
  │
  ├──── Outline                         ◄── PdfGlobal / PdfObject
  │         └── TocPrinter / tocstylesheet.cc
  │
  └──── QPrinter / QPainter  (Qt)
            └── MyLooksStyle  ◄── SVG form element rendering
```

---

## C Public API

The library exposes a C interface for embedding in other applications.

| File | Wraps |
|------|-------|
| `src/lib/pdf.h` | `PdfConverter` and `PdfGlobal`/`PdfObject` settings |
| `src/lib/pdf_c_bindings.cc` | C function implementations for PDF |
| `src/lib/image.h` | `ImageConverter` and `ImageGlobal` settings |
| `src/lib/image_c_bindings.cc` | C function implementations for image |

The C API follows a create/set/convert/destroy lifecycle:

```c
wkhtmltopdf_global_settings * gs = wkhtmltopdf_create_global_settings();
wkhtmltopdf_set_global_setting(gs, "out", "/tmp/out.pdf");
wkhtmltopdf_object_settings * os = wkhtmltopdf_create_object_settings();
wkhtmltopdf_set_object_setting(os, "page", "https://example.com");
wkhtmltopdf_converter * conv = wkhtmltopdf_create_converter(gs);
wkhtmltopdf_add_object(conv, os, NULL);
wkhtmltopdf_convert(conv);
wkhtmltopdf_destroy_converter(conv);
```

The settings `get`/`set` methods are backed by the reflection system in `src/lib/reflect.hh`, which maps string keys to struct fields.

---

## Key Qt Build Flags

| Flag | Effect |
|------|--------|
| `__EXTENSIVE_WKHTMLTOPDF_QT_HACK__` | Enables TOC (`Outline`), link resolution, header/footer height measurement, and `QWebPrinter` integration. Requires the patched Qt in `qt/`. |
| `DLL_PUBLIC` / `DLL_LOCAL` | Symbol visibility macros (defined in `dllbegin.inc`/`dllend.inc`) |
