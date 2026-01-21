#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QIcon>
#include <QQuickStyle>

// cxx-qt generated bridges
extern "C" bool cxx_qt_init_crate_myme_ui();

// Rust initialization function
extern "C" bool initialize_note_model(const char* base_url);

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    // Set application metadata
    app.setApplicationName("MyMe");
    app.setOrganizationName("MyMe");
    app.setApplicationVersion("0.1.0");

    // Set icon theme (for Kirigami icons)
    QIcon::setThemeName("breeze");

    // Use Basic style to allow QML customization of controls
    QQuickStyle::setStyle("Basic");

    // Initialize cxx-qt types (this also registers QML types)
    cxx_qt_init_crate_myme_ui();

    // Initialize global services (TodoClient, etc.)
    initialize_note_model("http://localhost:8008");

    QQmlApplicationEngine engine;

    // Add QML import paths
    engine.addImportPath(":/qt/qml");
    engine.addImportPath("qrc:/");

    // Load main QML file
    const QUrl url(QStringLiteral("qrc:/crates/myme-ui/qml/Main.qml"));
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreated,
        &app,
        [url](QObject *obj, const QUrl &objUrl) {
            if (!obj && url == objUrl)
                QCoreApplication::exit(-1);
        },
        Qt::QueuedConnection
    );

    engine.load(url);

    if (engine.rootObjects().isEmpty())
        return -1;

    return app.exec();
}
