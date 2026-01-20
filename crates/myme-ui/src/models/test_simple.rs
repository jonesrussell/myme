use core::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type TestObject = super::TestObjectRust;

        #[qinvokable]
        fn test_method(self: Pin<&mut Self>);
    }
}

#[derive(Default)]
pub struct TestObjectRust;

impl qobject::TestObject {
    pub fn test_method(self: Pin<&mut Self>) {
        println!("Test method called");
    }
}
