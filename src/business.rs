use ic_canister_kit::common::once::call_once_guard;
#[allow(unused)]
use ic_canister_kit::identity::caller;

#[allow(unused)]
use crate::stable::*;
#[allow(unused)]
use crate::types::*;

// 查询
#[ic_cdk::query(guard = "has_business_query")]
fn business_files() -> Vec<QueryFile> {
    with_state(|s| s.business_files())
}

#[ic_cdk::query(guard = "has_business_query")]
fn business_download(path: String) -> Vec<u8> {
    with_state(|s| s.business_download(path))
}

// 下载数据数据
#[ic_cdk::query(guard = "has_business_query")]
fn business_download_by(path: String, offset: u64, offset_end: u64) -> Vec<u8> {
    with_state(|s| s.business_download_by(path, offset, offset_end))
}

// 修改
#[ic_cdk::update(guard = "has_business_upload")]
fn business_upload(args: Vec<UploadingArg>) {
    let _guard = call_once_guard(); // post 接口应该拦截

    let caller = caller();
    let arg_content = format!(
        "upload file: [{}]",
        args.iter()
            .map(|arg| format!("path: {} size: {} index: {}", arg.path, arg.size, arg.index))
            .collect::<Vec<_>>()
            .join(", ")
    ); // * 记录参数内容

    with_mut_state(
        |s| {
            s.business_upload(args);
            (None, ())
        },
        caller,
        RecordTopics::UploadFile.topic(),
        arg_content,
    )
}

#[ic_cdk::update(guard = "has_business_delete")]
fn business_delete(names: Vec<String>) {
    let _guard = call_once_guard(); // post 接口应该拦截

    let caller = caller();
    let arg_content = format!("delete file: [{}]", &names.join(", ")); // * 记录参数内容

    with_mut_state(
        |s: &mut State| {
            s.business_delete(names);
            (None, ())
        },
        caller,
        RecordTopics::DeleteFile.topic(),
        arg_content,
    )
}
