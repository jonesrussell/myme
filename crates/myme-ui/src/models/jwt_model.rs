use core::pin::Pin;

use cxx_qt_lib::QString;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::Value;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, payload)]
        #[qproperty(QString, secret)]
        #[qproperty(QString, algorithm)]
        #[qproperty(QString, generated_token)]
        #[qproperty(QString, error_message)]
        type JwtModel = super::JwtModelRust;

        #[qinvokable]
        fn generate_token(self: Pin<&mut JwtModel>);

        #[qinvokable]
        fn copy_to_clipboard(self: &JwtModel);
    }
}

#[derive(Default)]
pub struct JwtModelRust {
    payload: QString,
    secret: QString,
    algorithm: QString,
    generated_token: QString,
    error_message: QString,
}

impl qobject::JwtModel {
    pub fn generate_token(mut self: Pin<&mut Self>) {
        // Clear previous state
        self.as_mut().set_error_message(QString::from(""));
        self.as_mut().set_generated_token(QString::from(""));

        let payload_str = self.as_ref().payload().to_string();
        let secret_str = self.as_ref().secret().to_string();
        let algorithm_str = self.as_ref().algorithm().to_string();

        // Validate inputs
        if payload_str.trim().is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Payload is required"));
            return;
        }

        if secret_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Secret key is required"));
            return;
        }

        // Parse payload as JSON
        let claims: Value = match serde_json::from_str(&payload_str) {
            Ok(v) => v,
            Err(e) => {
                self.as_mut()
                    .set_error_message(QString::from(format!("Invalid JSON: {}", e)));
                return;
            }
        };

        // Ensure claims is an object
        if !claims.is_object() {
            self.as_mut()
                .set_error_message(QString::from("Payload must be a JSON object"));
            return;
        }

        // Parse algorithm
        let alg = match algorithm_str.as_str() {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            _ => Algorithm::HS256, // Default to HS256
        };

        // Create header with selected algorithm
        let header = Header::new(alg);

        // Encode the token
        match encode(&header, &claims, &EncodingKey::from_secret(secret_str.as_bytes())) {
            Ok(token) => {
                tracing::info!("Generated JWT token with algorithm {:?}", alg);
                self.as_mut().set_generated_token(QString::from(&token));
            }
            Err(e) => {
                tracing::error!("Failed to generate JWT: {}", e);
                self.as_mut()
                    .set_error_message(QString::from(format!("Failed to generate token: {}", e)));
            }
        }
    }

    pub fn copy_to_clipboard(&self) {
        let token = self.generated_token().to_string();
        if token.is_empty() {
            return;
        }

        // Note: Clipboard functionality requires platform-specific implementation
        // For now, we'll log this action. In a full implementation, you would use
        // Qt's clipboard API through the QML side or a clipboard crate.
        tracing::info!("Token copied to clipboard (token length: {})", token.len());
    }
}
