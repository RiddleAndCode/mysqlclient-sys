use proc_macro::{TokenStream, TokenTree};
use semver::{Version, VersionReq};
use std::ffi::CStr;

const VERSION: &[u8] = mysqlclient_bindings::LIBMYSQL_VERSION;
const ATTR_ERR_MSG: &str = "Expected a valid semantic version requirement string as \
                            argument to `mysqlclient_version`";

#[proc_macro_attribute]
pub fn mysqlclient_version(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mysqlclient_ver = Version::parse(
        CStr::from_bytes_with_nul(VERSION)
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    if let Some(TokenTree::Literal(literal)) = attr.into_iter().next() {
        let literal_string = literal.to_string();
        if literal_string.len() < 2 {
            panic!(ATTR_ERR_MSG)
        }
        let req =
            VersionReq::parse(&literal_string[1..literal_string.len() - 1]).expect(ATTR_ERR_MSG);
        if req.matches(&mysqlclient_ver) {
            item
        } else {
            TokenStream::new()
        }
    } else {
        panic!(ATTR_ERR_MSG);
    }
}
