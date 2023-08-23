use regex::Regex;
use std::{borrow::Cow, collections::HashMap};

use percent_encoding::percent_decode_str;

use crate::stable::State;

use super::types::*;

// 为了自动生成 did，这个方法仅仅为了占位
#[candid::candid_method(query, rename = "http_request")]
fn __http_request(_req: CustomHttpRequest) -> CustomHttpResponse<'static> {
    todo!()
}

// 请求 nft 数据
// This could reply with a lot of data. To return this data from the function would require it to be cloned,
// because the thread_local! closure prevents us from returning data borrowed from inside it.
// Luckily, it doesn't actually get returned from the exported WASM function, that's just an abstraction.
// What happens is it gets fed to call::reply, and we can do that explicitly to save the cost of cloning the data.
// #[query] calls call::reply unconditionally, and calling it twice would trap, so we use #[export_name] directly.
// This requires duplicating the rest of the abstraction #[query] provides for us, like setting up the panic handler with
// ic_cdk::setup() and fetching the function parameters via call::arg_data.
// cdk 0.5 makes this unnecessary, but it has not been released at the time of writing this example.
// #[ic_cdk::query(name = "http_request")] // 这种写法不行，总是报错
#[export_name = "canister_query http_request"] // 必须这种写法
fn http_request() {
    ic_cdk::setup();
    let req = ic_cdk::api::call::arg_data::<(CustomHttpRequest,)>().0; // 取得请求参数，也就是请求体
    crate::stable::with_state(|_state| _http_request(req, _state));
}

#[inline]
fn _http_request(req: CustomHttpRequest, state: &State) {
    // ic_cdk::println!("request =============== ");

    let mut split_url = req.url.split('?');

    let path = split_url.next().unwrap_or("/"); // 分割出 url，默认是 /

    ic_cdk::println!("path: {:?} -> {}", req.url, path);

    let mut headers: HashMap<&str, Cow<str>> = HashMap::new();

    let mut body: Vec<u8> = format!("Total NFTs: {}", 123).into_bytes();
    let mut code = 200; // 响应码默认是 200

    // 根据路径找文件
    let file = state.assets.files.get(path);
    if let Some(file) = file {
        // 有对应的文件
    } else {
        body = not_found(&mut code, &mut headers);
    }

    ic_cdk::api::call::reply((CustomHttpResponse {
        status_code: code,
        headers,
        body: body.into(),
    },));
}

// 找不到对应的文件
fn not_found<'a>(code: &mut u16, headers: &mut HashMap<&'a str, Cow<'a, str>>) -> Vec<u8> {
    *code = 404;

    headers.insert("Content-Type", "text/plain".into());

    b"Not found"[..].into()
}
