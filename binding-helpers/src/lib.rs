extern crate pkg_config;

#[cfg(target_env = "msvc")]
extern crate vcpkg;

use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn probe_libs(should_link: bool) -> (Vec<String>, Vec<String>) {
    let (link_paths, include_paths, needs_link) =
    // try environment variables
    if let (Ok(link_paths), Ok(include_paths)) = (
        env::var("MYSQLCLIENT_LIB_DIR"),
        env::var("MYSQLCLIENT_INCLUDE_DIR"),
    ) {
        (vec![link_paths], vec![include_paths], should_link)
    // try pkg-config
    } else if let Ok((link_paths, include_paths)) = try_pkg_config(should_link) {
        (link_paths, include_paths, false)
    // try vcpkg
    } else if let Ok((link_paths, include_paths)) = try_vcpkg(should_link) {
        (link_paths, include_paths, false)
    // try mysql_config
    } else if let (Some(link_paths), Some(include_paths)) = (
        mysql_config_variable("pkglibdir"),
        mysql_config_variable("pkgincludedir"),
    ) {
        (vec![link_paths], vec![include_paths], should_link)
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
            println!("cargo:rustc-link-lib=ssl");
            println!("cargo:rustc-link-lib=crypto");
        }
    }

    (link_paths, include_paths)
}

pub fn bindings_builder(include_paths: Vec<String>, emit: bool) -> bindgen::Builder {
    let wrapper_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../binding-helpers/wrapper.h")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();

    if emit {
        println!("cargo:rerun-if-changed={}", wrapper_path);
    }

    let mut builder = bindgen::Builder::default()
        .header(wrapper_path)
        .blacklist_type("my_bool")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .default_alias_style(bindgen::AliasVariation::TypeAlias);

    for path in include_paths {
        builder = builder.clang_arg(&format!("-I{}", path));
    }

    if emit {
        builder = builder.parse_callbacks(Box::new(bindgen::CargoCallbacks));
    }

    builder
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

fn try_pkg_config(should_link: bool) -> Result<(Vec<String>, Vec<String>), pkg_config::Error> {
    pkg_config::Config::new()
        .print_system_cflags(should_link)
        .print_system_libs(should_link)
        .probe("mysqlclient")
        .map(|lib| (path_strs(lib.link_paths), path_strs(lib.include_paths)))
}

#[cfg(target_env = "msvc")]
fn try_vcpkg(should_link: bool) -> Result<(Vec<String>, Vec<String>), vcpkg::Error> {
    vcpkg::Config::new()
        .emit_includes(should_link)
        .find_package("libmysql")
        .map(|lib| (path_strs(lib.link_paths), path_strs(lib.include_paths)))
}

#[cfg(not(target_env = "msvc"))]
fn try_vcpkg(_: bool) -> Result<(Vec<String>, Vec<String>), ()> {
    Err(())
}
