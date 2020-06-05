extern crate pkg_config;

#[cfg(target_env = "msvc")]
extern crate vcpkg;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let (_, include_paths) = link_libs();
    build_bindings(include_paths);
}

fn build_bindings(include_paths: Vec<String>) {
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .default_alias_style(bindgen::AliasVariation::TypeAlias);

    for path in include_paths {
        builder = builder.clang_arg(&format!("-I{}", path));
    }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn link_libs() -> (Vec<String>, Vec<String>) {
    let (link_paths, include_paths, needs_link) =
    // try environment variables
    if let (Ok(link_paths), Ok(include_paths)) = (
        env::var("MYSQLCLIENT_LIB_DIR"),
        env::var("MYSQLCLIENT_INCLUDE_DIR"),
    ) {
        (vec![link_paths], vec![include_paths], true)
    // try pkg-config
    } else if let Ok((link_paths, include_paths)) = try_pkg_config() {
        (link_paths, include_paths, false)
    // try vcpkg
    } else if let Ok((link_paths, include_paths)) = try_vcpkg() {
        (link_paths, include_paths, false)
    // try mysql_config
    } else if let (Some(link_paths), Some(include_paths)) = (
        mysql_config_variable("pkglibdir"),
        mysql_config_variable("pkgincludedir"),
    ) {
        (vec![link_paths], vec![include_paths], true)
    } else {
        panic!("Could not find `mysqlclient` lib. \
                Either `pgk-config`, `vcpkg` or `mysql_config` needs to be installed \
                or the environment flags `MYSQLCLIENT_LIB_DIR` and `MYSQLCLIENT_INCLUDE_DIR` need to be set.")
    };

    if needs_link {
        for path in link_paths.iter() {
            println!("cargo:rustc-link-search=native={}", path);
        }
        if cfg!(all(windows, target_env = "gnu")) {
            println!("cargo:rustc-link-lib=dylib=mysql");
        } else if cfg!(all(windows, target_env = "msvc")) {
            println!("cargo:rustc-link-lib=static=mysqlclient");
        } else {
            println!("cargo:rustc-link-lib=mysqlclient");
        }
    }

    (link_paths, include_paths)
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

fn path_strs(paths: Vec<PathBuf>) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.as_os_str().to_string_lossy().to_string())
        .collect()
}

fn try_pkg_config() -> Result<(Vec<String>, Vec<String>), pkg_config::Error> {
    pkg_config::probe_library("mysqlclient")
        .map(|lib| (path_strs(lib.link_paths), path_strs(lib.include_paths)))
}

#[cfg(target_env = "msvc")]
fn try_vcpkg() -> Result<(Vec<String>, Vec<String>), vcpkg::Error> {
    vcpkg::find_package("libmysql")
        .map(|lib| (path_strs(lib.link_paths), path_strs(lib.include_paths)))
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg() -> Result<(Vec<String>, Vec<String>), ()> {
    Err(())
}
