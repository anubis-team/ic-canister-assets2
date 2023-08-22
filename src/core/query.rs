use crate::stable::{with_state, State};

use super::types::QueryFile;

#[ic_cdk::query(name = "files")]
#[candid::candid_method(query, rename = "files")]
fn files() -> Vec<QueryFile> {
    with_state(|s: &State| s.assets.files())
}
