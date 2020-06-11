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
    let version = version_callback.version().unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(
        out_path.join("version.rs"),
        format!("const VERSION: &str = \"{}\";", version),
    )
    .unwrap();
}
