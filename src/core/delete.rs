use crate::stable::{is_admin, must_be_running, with_mut_state, State};

// 删除文件
#[ic_cdk::update(name = "delete", guard = "is_admin")]
#[candid::candid_method(update, rename = "delete")]
fn delete(names: Vec<String>) {
    // ! 维护拦截检查
    must_be_running();

    with_mut_state(|s: &mut State| {
        for name in names {
            s.uploading.clean(&name);
            s.assets.clean(&name);
        }
    })
}
