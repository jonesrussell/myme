fn main() {
    // Build the cxx-qt bridge
    cxx_qt_build::CxxQtBuilder::new()
        .qml_module("com.myme", "1.0")
        .file("src/models/todo_model.rs")
        .build();
}
