extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use semver::VersionReq;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

const ATTR_ERR_MSG: &str = "Expected a valid semantic version requirement string as \
                            argument to `mysqlclient_version`";

#[proc_macro_attribute]
pub fn mysqlclient_version(attr: TokenStream, item: TokenStream) -> TokenStream {
    let version = VERSION
        .parse()
        .expect("`mysql_get_client_info` did not return a parseable version");
    if let Some(TokenTree::Literal(literal)) = attr.into_iter().next() {
        let literal_string = literal.to_string();
        if literal_string.len() < 2 {
            panic!(ATTR_ERR_MSG)
        }
        let req =
            VersionReq::parse(&literal_string[1..literal_string.len() - 1]).expect(ATTR_ERR_MSG);
        if req.matches(&version) {
            item
        } else {
            TokenStream::new()
        }
    } else {
        panic!(ATTR_ERR_MSG);
    }
}
