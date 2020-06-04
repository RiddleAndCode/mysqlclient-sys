extern crate pkg_config;

#[cfg(target_env = "msvc")]
extern crate vcpkg;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let (_, include_path) = link_libs();
    build_bindings(include_path);
}

fn build_bindings(include_path: impl AsRef<Path>) {
    let bindings = bindgen::Builder::default()
        .header(
            include_path
                .as_ref()
                .join("mysql.h")
                .as_os_str()
                .to_string_lossy(),
        )
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .default_alias_style(bindgen::AliasVariation::TypeAlias)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn link_libs() -> (String, String) {
    let (libs_path, include_path, needs_link) =
    // try environment variables
    if let (Ok(libs_path), Ok(include_path)) = (
        env::var("MYSQLCLIENT_LIB_DIR"),
        env::var("MYSQLCLIENT_INCLUDE_DIR"),
    ) {
        (libs_path, include_path, true)
    // try pkg-config
    } else if let Some((libs_path, include_path)) = try_pkg_config() {
        (libs_path, include_path, false)
    // try vcpkg
    } else if let Some((libs_path, include_path)) = try_vcpkg() {
        (libs_path, include_path, false)
    // try mysql_config
    } else if let (Some(libs_path), Some(include_path)) = (
        mysql_config_variable("pkglibdir"),
        mysql_config_variable("pkgincludedir"),
    ) {
        (libs_path, include_path, true)
    } else {
        panic!("Could not find `mysqlclient` lib. \
                Either `pgk-config`, `vcpkg` or `mysql_config` needs to be installed \
                or the environment flags `MYSQLCLIENT_LIB_DIR` and `MYSQLCLIENT_INCLUDE_DIR` need to be set.")
    };

    if needs_link {
        println!("cargo:rustc-link-search=native={}", libs_path);
        if cfg!(all(windows, target_env = "gnu")) {
            println!("cargo:rustc-link-lib=dylib=mysql");
        } else if cfg!(all(windows, target_env = "msvc")) {
            println!("cargo:rustc-link-lib=static=mysqlclient");
        } else {
            println!("cargo:rustc-link-lib=mysqlclient");
        }
    }

    (libs_path, include_path)
}

fn mysql_config_variable(var_name: &str) -> Option<String> {
    Command::new("mysql_config")
        .arg(format!("--variable={}", var_name))
        .output()
        .into_iter()
        .filter(|output| output.status.success())
        .flat_map(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .next()
}

fn extract_paths(
    link_paths: Vec<PathBuf>,
    include_paths: Vec<PathBuf>,
) -> Option<(String, String)> {
    eprintln!("{:?} {:?}", link_paths, include_paths);
    link_paths
        .get(0)
        .and_then(|link_path| {
            include_paths
                .get(0)
                .and_then(|include_path| Some((link_path, include_path)))
        })
        .map(|(link_path, include_path)| {
            (
                link_path.as_os_str().to_string_lossy().to_string(),
                include_path.as_os_str().to_string_lossy().to_string(),
            )
        })
}

fn try_pkg_config() -> Option<(String, String)> {
    pkg_config::probe_library("mysqlclient")
        .ok()
        .and_then(|lib| extract_paths(lib.link_paths, lib.include_paths))
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> Option<(String, String)> {
    vcpkg::find_package("libmysql")
        .ok()
        .and_then(|lib| extract_paths(lib.link_paths, lib.include_paths))
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> Option<(String, String)> {
    None
}
