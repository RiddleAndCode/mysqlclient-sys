use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{env, fs};

#[derive(Clone, Debug, Default)]
pub struct MysqlVersionCallback(Arc<Mutex<Option<String>>>);

impl ParseCallbacks for MysqlVersionCallback {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        match name {
            "MYSQL_SERVER_VERSION" => MacroParsingBehavior::Default,
            _ => MacroParsingBehavior::Ignore,
        }
    }
    fn str_macro(&self, name: &str, value: &[u8]) {
        match name {
            "MYSQL_SERVER_VERSION" => {
                let mut version = self.0.lock().unwrap();
                *version = Some(String::from_utf8_lossy(value).to_string());
            }
            _ => (),
        }
    }
}

impl MysqlVersionCallback {
    fn version(&self) -> Option<String> {
        self.0.lock().unwrap().clone()
    }
}

fn main() {
    let version_callback = MysqlVersionCallback::default();
    let mut builder =
        binding_helpers::bindings_builder(vec!["/usr/include/mysql".to_string()], true);
    builder = builder.parse_callbacks(Box::new(version_callback.clone()));
    builder.generate().unwrap();

    let version_str = version_callback.version().unwrap();
    let version_parts: Vec<&str> = version_str.split("-").collect();
    let version = version_parts[0];
    let vendor = match version_parts.get(1) {
        Some(_) => "Vendor::MariaDB",
        None => "Vendor::MySQL",
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out_path.join("version.rs"),
        format!(
            r#"
            const VERSION: &str = "{}";
            const VENDOR: Vendor = {};
        "#,
            version, vendor
        ),
    )
    .unwrap();
}
