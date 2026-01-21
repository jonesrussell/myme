use core::pin::Pin;

use cxx_qt_lib::{QString, QStringList};
use uuid::Uuid;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qstringlist.h");
        type QString = cxx_qt_lib::QString;
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, version)]
        #[qproperty(i32, count)]
        #[qproperty(QString, format)]
        #[qproperty(QString, namespace_type)]
        #[qproperty(QString, custom_namespace)]
        #[qproperty(QString, name)]
        #[qproperty(QStringList, generated_uuids)]
        #[qproperty(QString, error_message)]
        type UuidModel = super::UuidModelRust;

        #[qinvokable]
        fn generate(self: Pin<&mut UuidModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut UuidModel>);
    }
}

pub struct UuidModelRust {
    version: QString,
    count: i32,
    format: QString,
    namespace_type: QString,
    custom_namespace: QString,
    name: QString,
    generated_uuids: QStringList,
    error_message: QString,
}

impl Default for UuidModelRust {
    fn default() -> Self {
        Self {
            version: QString::from("v4"),
            count: 1,
            format: QString::from("standard"),
            namespace_type: QString::from("dns"),
            custom_namespace: QString::from(""),
            name: QString::from(""),
            generated_uuids: QStringList::default(),
            error_message: QString::from(""),
        }
    }
}

impl qobject::UuidModel {
    pub fn generate(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));

        let version = self.as_ref().version().to_string();
        let count = (*self.as_ref().count()).max(1).min(100);
        let format = self.as_ref().format().to_string();
        let namespace_type = self.as_ref().namespace_type().to_string();
        let custom_ns = self.as_ref().custom_namespace().to_string();
        let name = self.as_ref().name().to_string();

        let mut uuids = QStringList::default();

        for _ in 0..count {
            let uuid_result = match version.as_str() {
                "v1" => Ok(Uuid::now_v1(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55])),
                "v4" => Ok(Uuid::new_v4()),
                "v5" => {
                    if name.is_empty() {
                        Err("Name is required for UUID v5".to_string())
                    } else {
                        let ns = match namespace_type.as_str() {
                            "dns" => Uuid::NAMESPACE_DNS,
                            "url" => Uuid::NAMESPACE_URL,
                            "oid" => Uuid::NAMESPACE_OID,
                            "x500" => Uuid::NAMESPACE_X500,
                            "custom" => match Uuid::parse_str(&custom_ns) {
                                Ok(u) => u,
                                Err(e) => {
                                    self.as_mut().set_error_message(QString::from(&format!(
                                        "Invalid custom namespace UUID: {}",
                                        e
                                    )));
                                    return;
                                }
                            },
                            _ => Uuid::NAMESPACE_DNS,
                        };
                        Ok(Uuid::new_v5(&ns, name.as_bytes()))
                    }
                }
                "v7" => Ok(Uuid::now_v7()),
                _ => Ok(Uuid::new_v4()),
            };

            match uuid_result {
                Ok(u) => {
                    let formatted = Self::format_uuid(&u, &format);
                    uuids.append(QString::from(&formatted));
                }
                Err(e) => {
                    self.as_mut().set_error_message(QString::from(&e));
                    return;
                }
            }
        }

        self.as_mut().set_generated_uuids(uuids);
    }

    fn format_uuid(uuid: &Uuid, format: &str) -> String {
        match format {
            "no-dashes" => uuid.simple().to_string(),
            "uppercase" => uuid.to_string().to_uppercase(),
            "braces" => format!("{{{}}}", uuid),
            _ => uuid.to_string(),
        }
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_generated_uuids(QStringList::default());
        self.as_mut().set_error_message(QString::from(""));
    }
}
