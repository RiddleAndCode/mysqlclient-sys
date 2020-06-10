extern crate proc_macro;

use mysqlclient_bindings as bindings;
use proc_macro::{TokenStream, TokenTree};
use semver::{Version, VersionReq};
use std::ffi::CStr;

const ATTR_ERR_MSG: &str = "Expected a valid semantic version requirement string as \
                            argument to `mysqlclient_version`";

fn get_version() -> Version {
    unsafe { CStr::from_ptr(bindings::mysql_get_client_info()) }
        .to_str()
        .unwrap()
        .parse()
        .expect("`mysql_get_client_info` did not return a parseable version")
}

#[proc_macro_attribute]
pub fn mysqlclient_version(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(TokenTree::Literal(literal)) = attr.into_iter().next() {
        let literal_string = literal.to_string();
        if literal_string.len() < 2 {
            panic!(ATTR_ERR_MSG)
        }
        let req =
            VersionReq::parse(&literal_string[1..literal_string.len() - 1]).expect(ATTR_ERR_MSG);
        if req.matches(&get_version()) {
            item
        } else {
            TokenStream::new()
        }
    } else {
        panic!(ATTR_ERR_MSG);
    }
}
