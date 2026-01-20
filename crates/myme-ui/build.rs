fn main() {
    cxx_qt_build::CxxQtBuilder::new()
        .file("src/models/note_model.rs")
        .file("src/models/repo_model.rs")
        .build();
}
