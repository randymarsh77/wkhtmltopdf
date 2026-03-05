// webkit_renderer.cpp – Qt WebEngine rendering implementation.
//
// This file implements the `wkhtmltopdf::render_url` function declared in
// `webkit_renderer.h`.  It uses QWebEngineView to load the requested URL,
// waits for the page to finish loading (with an optional JS-settle delay),
// evaluates any requested JavaScript snippets via QWebEnginePage::runJavaScript,
// then grabs a screenshot and returns it as PNG bytes.
//
// Requirements:
//   Qt 5.6+ or Qt 6 with the WebEngineWidgets module.
//   Must be compiled with the Qt WebEngineWidgets module linked.

#include "wkhtmltopdf/webkit_renderer.h"

#include <QApplication>
#include <QBuffer>
#include <QByteArray>
#include <QEventLoop>
#include <QImage>
#include <QPixmap>
#include <QString>
#include <QTimer>
#include <QUrl>
#include <QVariant>
#include <QWebEnginePage>
#include <QWebEngineSettings>
#include <QWebEngineView>

#include <algorithm>
#include <iterator>
#include <memory>
#include <stdexcept>
#include <string>

namespace wkhtmltopdf {

rust::Vec<uint8_t> render_url(rust::Str url, bool js_enabled,
                               uint32_t js_delay_ms,
                               rust::Slice<rust::Str> run_scripts) {
    // Ensure a QApplication exists for the Qt event loop.  If the calling
    // process already created one we reuse it; otherwise we own one for the
    // duration of this call.
    static int qt_argc = 0;
    std::unique_ptr<QApplication> owned_app;
    if (QApplication::instance() == nullptr) {
        owned_app = std::make_unique<QApplication>(qt_argc, nullptr);
    }

    QWebEngineView view;

    // Apply JavaScript settings.
    QWebEngineSettings *settings = view.page()->settings();
    settings->setAttribute(QWebEngineSettings::JavascriptEnabled, js_enabled);

    // Set a sensible default viewport size.
    view.resize(1024, 768);

    // Connect the loadFinished signal to quit the event loop, optionally
    // honouring a post-load JS-settle delay.
    QEventLoop loop;
    bool page_load_ok = false;

    QObject::connect(
        &view, &QWebEngineView::loadFinished, [&](bool ok) {
            page_load_ok = ok;
            if (js_delay_ms > 0) {
                QTimer::singleShot(static_cast<int>(js_delay_ms), &loop,
                                   &QEventLoop::quit);
            } else {
                loop.quit();
            }
        });

    // Build a null-terminated QString from the Rust string slice.
    const QString qurl =
        QString::fromUtf8(url.data(), static_cast<int>(url.size()));
    view.load(QUrl(qurl));
    loop.exec();

    if (!page_load_ok) {
        const std::string msg =
            "Qt WebEngine: page load failed for URL: " +
            std::string(url.data(), url.size());
        throw std::runtime_error(msg);
    }

    // Execute any user-provided JavaScript snippets sequentially.  Each
    // script is run via QWebEnginePage::runJavaScript; a local QEventLoop
    // is used to block until the asynchronous callback fires, making the
    // execution effectively synchronous from the caller's perspective.
    //
    // Note: Qt WebEngine's runJavaScript callback receives the script's
    // return value as a QVariant but does not surface JavaScript runtime
    // errors (exceptions are swallowed by the engine).  The flag guard
    // below prevents a stall if the callback fires before exec() is reached.
    if (js_enabled && !run_scripts.empty()) {
        QWebEnginePage *page = view.page();
        for (const rust::Str &script : run_scripts) {
            const QString qscript =
                QString::fromUtf8(script.data(), static_cast<int>(script.size()));
            QEventLoop script_loop;
            bool script_done = false;
            page->runJavaScript(qscript, [&script_loop, &script_done](const QVariant &) {
                script_done = true;
                script_loop.quit();
            });
            if (!script_done) {
                script_loop.exec();
            }
        }
    }

    // Grab the rendered viewport as a QPixmap and encode to PNG.
    const QPixmap pixmap = view.grab();
    QByteArray png_bytes;
    {
        QBuffer buffer(&png_bytes);
        buffer.open(QIODevice::WriteOnly);
        pixmap.save(&buffer, "PNG");
    }

    // Copy the PNG bytes into a rust::Vec<uint8_t>.
    rust::Vec<uint8_t> result;
    result.reserve(static_cast<std::size_t>(png_bytes.size()));
    const auto *data =
        reinterpret_cast<const uint8_t *>(png_bytes.constData());
    std::copy(data, data + png_bytes.size(), std::back_inserter(result));
    return result;
}

} // namespace wkhtmltopdf
