use core::pin::Pin;

use cxx_qt_lib::QString;
use digest::Digest;
use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

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
        #[qproperty(QString, input_type)]
        #[qproperty(QString, file_path)]
        #[qproperty(QString, file_name)]
        #[qproperty(i64, file_size)]
        #[qproperty(QString, hash_md5)]
        #[qproperty(QString, hash_sha1)]
        #[qproperty(QString, hash_sha256)]
        #[qproperty(QString, hash_sha512)]
        #[qproperty(bool, hmac_enabled)]
        #[qproperty(QString, hmac_key)]
        #[qproperty(QString, hmac_algorithm)]
        #[qproperty(QString, hmac_result)]
        #[qproperty(QString, compare_hash)]
        #[qproperty(QString, compare_result)]
        #[qproperty(QString, error_message)]
        type HashModel = super::HashModelRust;

        #[qinvokable]
        fn hash_text(self: Pin<&mut HashModel>);

        #[qinvokable]
        fn hash_file(self: Pin<&mut HashModel>, path: QString);

        #[qinvokable]
        fn compute_hmac(self: Pin<&mut HashModel>);

        #[qinvokable]
        fn compare(self: Pin<&mut HashModel>);

        #[qinvokable]
        fn export_checksums(self: &HashModel) -> QString;

        #[qinvokable]
        fn clear(self: Pin<&mut HashModel>);
    }
}

pub struct HashModelRust {
    input: QString,
    input_type: QString,
    file_path: QString,
    file_name: QString,
    file_size: i64,
    hash_md5: QString,
    hash_sha1: QString,
    hash_sha256: QString,
    hash_sha512: QString,
    hmac_enabled: bool,
    hmac_key: QString,
    hmac_algorithm: QString,
    hmac_result: QString,
    compare_hash: QString,
    compare_result: QString,
    error_message: QString,
}

impl Default for HashModelRust {
    fn default() -> Self {
        Self {
            input: QString::from(""),
            input_type: QString::from("text"),
            file_path: QString::from(""),
            file_name: QString::from(""),
            file_size: 0,
            hash_md5: QString::from(""),
            hash_sha1: QString::from(""),
            hash_sha256: QString::from(""),
            hash_sha512: QString::from(""),
            hmac_enabled: false,
            hmac_key: QString::from(""),
            hmac_algorithm: QString::from("sha256"),
            hmac_result: QString::from(""),
            compare_hash: QString::from(""),
            compare_result: QString::from(""),
            error_message: QString::from(""),
        }
    }
}

impl qobject::HashModel {
    pub fn hash_text(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();

        if input.is_empty() {
            self.as_mut().clear_hashes();
            return;
        }

        let bytes = input.as_bytes();
        self.as_mut().compute_all_hashes(bytes);
    }

    pub fn hash_file(mut self: Pin<&mut Self>, path: QString) {
        self.as_mut().set_error_message(QString::from(""));
        let path_str = path.to_string();

        match std::fs::read(&path_str) {
            Ok(bytes) => {
                let file_name = std::path::Path::new(&path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                self.as_mut().set_file_path(path);
                self.as_mut().set_file_name(QString::from(&file_name));
                self.as_mut().set_file_size(bytes.len() as i64);
                self.as_mut().compute_all_hashes(&bytes);
            }
            Err(e) => {
                self.as_mut()
                    .set_error_message(QString::from(&format!("Failed to read file: {}", e)));
            }
        }
    }

    fn compute_all_hashes(mut self: Pin<&mut Self>, bytes: &[u8]) {
        let md5 = hex::encode(Md5::digest(bytes));
        let sha1 = hex::encode(Sha1::digest(bytes));
        let sha256 = hex::encode(Sha256::digest(bytes));
        let sha512 = hex::encode(Sha512::digest(bytes));

        self.as_mut().set_hash_md5(QString::from(&md5));
        self.as_mut().set_hash_sha1(QString::from(&sha1));
        self.as_mut().set_hash_sha256(QString::from(&sha256));
        self.as_mut().set_hash_sha512(QString::from(&sha512));
    }

    fn clear_hashes(mut self: Pin<&mut Self>) {
        self.as_mut().set_hash_md5(QString::from(""));
        self.as_mut().set_hash_sha1(QString::from(""));
        self.as_mut().set_hash_sha256(QString::from(""));
        self.as_mut().set_hash_sha512(QString::from(""));
    }

    pub fn compute_hmac(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();
        let key = self.as_ref().hmac_key().to_string();
        let algorithm = self.as_ref().hmac_algorithm().to_string();

        if input.is_empty() || key.is_empty() {
            self.as_mut().set_hmac_result(QString::from(""));
            return;
        }

        let result = match algorithm.as_str() {
            "sha256" => {
                let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(input.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            }
            "sha512" => {
                let mut mac = Hmac::<Sha512>::new_from_slice(key.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(input.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            }
            _ => return,
        };

        self.as_mut().set_hmac_result(QString::from(&result));
    }

    pub fn compare(mut self: Pin<&mut Self>) {
        let compare = self
            .as_ref()
            .compare_hash()
            .to_string()
            .to_lowercase()
            .replace(" ", "");

        if compare.is_empty() {
            self.as_mut().set_compare_result(QString::from(""));
            return;
        }

        let hashes = [
            (self.as_ref().hash_md5().to_string(), "MD5"),
            (self.as_ref().hash_sha1().to_string(), "SHA-1"),
            (self.as_ref().hash_sha256().to_string(), "SHA-256"),
            (self.as_ref().hash_sha512().to_string(), "SHA-512"),
        ];

        for (hash, name) in hashes {
            if hash.to_lowercase() == compare {
                self.as_mut()
                    .set_compare_result(QString::from(&format!("match:{}", name)));
                return;
            }
        }

        self.as_mut().set_compare_result(QString::from("no-match"));
    }

    pub fn export_checksums(&self) -> QString {
        let mut output = String::new();
        let name = self.file_name().to_string();
        let name = if name.is_empty() { "input.txt" } else { &name };

        let sha256 = self.hash_sha256().to_string();
        if !sha256.is_empty() {
            output.push_str(&format!("{}  {}\n", sha256, name));
        }

        QString::from(&output)
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_input(QString::from(""));
        self.as_mut().set_file_path(QString::from(""));
        self.as_mut().set_file_name(QString::from(""));
        self.as_mut().set_file_size(0);
        self.as_mut().clear_hashes();
        self.as_mut().set_hmac_result(QString::from(""));
        self.as_mut().set_compare_hash(QString::from(""));
        self.as_mut().set_compare_result(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }
}
