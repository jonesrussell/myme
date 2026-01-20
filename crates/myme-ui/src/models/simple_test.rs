// Minimal cxx-qt test to verify macro expansion works

#[cxx_qt::bridge]
mod qobject {
    extern "RustQt" {
        #[qobject]
        type MyObject = super::MyObjectRust;
    }
}

#[derive(Default)]
pub struct MyObjectRust;
