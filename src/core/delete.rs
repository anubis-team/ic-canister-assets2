use crate::stable::{is_admin, with_mut_state, State};

// 删除文件
#[ic_cdk::update(name = "delete", guard = "is_admin")]
#[candid::candid_method(update, rename = "delete")]
fn delete(names: Vec<String>) {
    with_mut_state(|s: &mut State| {
        for name in names {
            s.uploading.clean(&name);
            s.assets.clean(&name);
        }
    })
}
