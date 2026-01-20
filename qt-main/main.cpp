#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QIcon>

// cxx-qt generated bridge (will be available after build)
// #include "cxxqt/todo_model.h"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);

    // Set application metadata
    app.setApplicationName("MyMe");
    app.setOrganizationName("MyMe");
    app.setApplicationVersion("0.1.0");

    // Set icon theme (for Kirigami icons)
    QIcon::setThemeName("breeze");

    QQmlApplicationEngine engine;

    // Register cxx-qt types
    // This will be uncommented once cxx-qt bridge is built
    // qmlRegisterType<TodoModel>("com.myme", 1, 0, "TodoModel");

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
