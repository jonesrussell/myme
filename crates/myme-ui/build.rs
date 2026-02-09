use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("myme_ui"))
        .file("src/models/auth_model.rs")
        .file("src/models/calendar_model.rs")
        .file("src/models/encoding_model.rs")
        .file("src/models/gmail_model.rs")
        .file("src/models/google_auth_model.rs")
        .file("src/models/hash_model.rs")
        .file("src/models/json_model.rs")
        .file("src/models/jwt_model.rs")
        .file("src/models/kanban_model.rs")
        .file("src/models/note_model.rs")
        .file("src/models/project_model.rs")
        .file("src/models/repo_model.rs")
        .file("src/models/workflow_model.rs")
        .file("src/models/time_model.rs")
        .file("src/models/uuid_model.rs")
        .file("src/models/weather_model.rs")
        .build();
}
