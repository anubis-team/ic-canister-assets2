use crate::stable::{is_admin, must_be_running, with_mut_state, State};

use super::types::UploadingArg;

// 上传数据
#[ic_cdk::update(name = "upload", guard = "is_admin")]
#[candid::candid_method(update, rename = "upload")]
fn upload(args: Vec<UploadingArg>) {
    // ! 维护拦截检查
    must_be_running();

    with_mut_state(|s: &mut State| {
        for arg in args {
            // 1. 先暂存本次内容
            let done = s.uploading.put(arg);

            if let Some(file) = done {
                // 2. 如果完成了, 需要升级本文件
                s.assets.put(&file);

                // 3. 删除这个文件
                let path = file.path.clone();
                s.uploading.clean(&path);
            }
        }
    })
}
