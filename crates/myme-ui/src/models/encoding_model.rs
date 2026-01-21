use core::pin::Pin;

use base64::{engine::general_purpose, Engine};
use cxx_qt_lib::QString;
use percent_encoding::{percent_decode_str, percent_encode, NON_ALPHANUMERIC};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, input)]
        #[qproperty(QString, output)]
        #[qproperty(QString, encoding_type)]
        #[qproperty(QString, error_message)]
        type EncodingModel = super::EncodingModelRust;

        #[qinvokable]
        fn encode(self: Pin<&mut EncodingModel>);

        #[qinvokable]
        fn decode(self: Pin<&mut EncodingModel>);

        #[qinvokable]
        fn swap(self: Pin<&mut EncodingModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut EncodingModel>);
    }
}

#[derive(Default)]
pub struct EncodingModelRust {
    input: QString,
    output: QString,
    encoding_type: QString,
    error_message: QString,
}

impl qobject::EncodingModel {
    pub fn encode(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        self.as_mut().set_output(QString::from(""));

        let input_str = self.as_ref().input().to_string();
        let encoding = self.as_ref().encoding_type().to_string();

        if input_str.is_empty() {
            return;
        }

        let result = match encoding.as_str() {
            "base64" => general_purpose::STANDARD.encode(input_str.as_bytes()),
            "base64url" => general_purpose::URL_SAFE.encode(input_str.as_bytes()),
            "hex" => hex::encode(input_str.as_bytes()),
            "url" => percent_encode(input_str.as_bytes(), NON_ALPHANUMERIC).to_string(),
            _ => general_purpose::STANDARD.encode(input_str.as_bytes()),
        };

        self.as_mut().set_output(QString::from(&result));
    }

    pub fn decode(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        self.as_mut().set_output(QString::from(""));

        let input_str = self.as_ref().input().to_string();
        let encoding = self.as_ref().encoding_type().to_string();

        if input_str.is_empty() {
            return;
        }

        let result = match encoding.as_str() {
            "base64" => match general_purpose::STANDARD.decode(input_str.trim()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                },
                Err(e) => Err(format!("Invalid Base64: {}", e)),
            },
            "base64url" => match general_purpose::URL_SAFE.decode(input_str.trim()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                },
                Err(e) => Err(format!("Invalid Base64 URL: {}", e)),
            },
            "hex" => match hex::decode(input_str.trim()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                },
                Err(e) => Err(format!("Invalid Hex: {}", e)),
            },
            "url" => match percent_decode_str(input_str.trim()).decode_utf8() {
                Ok(s) => Ok(s.to_string()),
                Err(e) => Err(format!("Invalid URL encoding: {}", e)),
            },
            _ => Err("Unknown encoding type".to_string()),
        };

        match result {
            Ok(decoded) => self.as_mut().set_output(QString::from(&decoded)),
            Err(e) => self.as_mut().set_error_message(QString::from(&e)),
        }
    }

    pub fn swap(mut self: Pin<&mut Self>) {
        let current_output = self.as_ref().output().to_string();
        self.as_mut().set_input(QString::from(&current_output));
        self.as_mut().set_output(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_input(QString::from(""));
        self.as_mut().set_output(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }
}
