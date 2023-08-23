use crate::stable::{with_state, State};

// 下载数据数据
#[ic_cdk::query(name = "download")]
#[candid::candid_method(query, rename = "download")]
fn download(path: String) -> Vec<u8> {
    with_state(|s: &State| s.assets.download(path))
}

// 下载数据数据
#[ic_cdk::query(name = "download_by")]
#[candid::candid_method(query, rename = "download_by")]
fn download_by(path: String, offset: u64, offset_end: u64) -> Vec<u8> {
    with_state(|s: &State| s.assets.download_by(path, offset, offset_end))
}
