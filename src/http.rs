use std::{borrow::Cow, collections::HashMap};

use percent_encoding::percent_decode_str;
use regex::Regex;

use ic_canister_kit::http::MAX_RESPONSE_LENGTH;

use crate::explore::explore;
use crate::stable::State;
use crate::types::*;

// https://github.com/dfinity/examples/blob/8b01d548d8548a9d4558a7a1dbb49234d02d7d03/motoko/http_counter/src/main.mo

// #[ic_cdk::update]
// fn http_request_update(request: CustomHttpRequest) -> CustomHttpResponse {
//     todo!()
// }

// 请求数据
#[ic_cdk::query]
fn http_request(request: CustomHttpRequest) -> CustomHttpResponse {
    crate::stable::with_state(|state| inner_http_request(state, request))
}

#[inline]
fn inner_http_request(state: &State, req: CustomHttpRequest) -> CustomHttpResponse {
    let mut split_url = req.url.split('?');
    let request_headers = req.headers;

    let path = split_url.next().unwrap_or("/"); // 分割出 url，默认是 /
    let path = percent_decode_str(path)
        .decode_utf8()
        .unwrap_or(Cow::Borrowed(path));
    let params = split_url.next().unwrap_or(""); // 请求参数
    let params = percent_decode_str(params)
        .decode_utf8()
        .unwrap_or(Cow::Borrowed(params));

    // ic_cdk::println!("============== path: {} -> {}", req.url, path);
    // for (key, value) in request_headers.iter() {
    //     ic_cdk::println!("header: {}: {}", key, value);
    // }

    let mut code = 200; // 响应码默认是 200
    let mut headers: HashMap<&str, Cow<str>> = HashMap::new();
    let body: Vec<u8>;
    let mut streaming_strategy: Option<StreamingStrategy> = None;

    if path == "/" {
        body = explore(&mut headers, state); // 主页内容
    } else {
        // 根据路径找文件
        let file = state.business_assets_get_file(path.as_ref());
        if let Some(file) = file {
            let asset = state.business_assets_get(&file.hash);
            if let Some(asset) = asset {
                let (_body, _streaming_strategy): (Vec<u8>, Option<StreamingStrategy>) = toast(
                    &path,
                    &params,
                    &request_headers,
                    file,
                    asset,
                    &mut code,
                    &mut headers,
                ); // 有对应的文件
                body = _body;
                streaming_strategy = _streaming_strategy;
            } else {
                body = not_found(&mut code, &mut headers);
            }
        } else {
            body = not_found(&mut code, &mut headers);
        }
    }

    CustomHttpResponse {
        status_code: code,
        headers: headers
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        body,
        streaming_strategy,
        upgrade: None,
    }
}

#[inline]
fn toast<'a>(
    path: &str,
    params: &str,
    request_headers: &HashMap<String, String>,
    file: &'a AssetFile,
    asset: &AssetData,
    code: &mut u16,
    headers: &mut HashMap<&'a str, Cow<'a, str>>,
) -> (Vec<u8>, Option<StreamingStrategy>) {
    // 1. 设置 header
    let (offset, size, streaming_strategy) = set_headers(
        path,
        params,
        request_headers,
        file,
        file.size as usize,
        code,
        headers,
    );

    // 2. 返回指定的内容
    (
        (asset.slice(&file.hash, file.size, offset, size)).to_vec(),
        streaming_strategy,
    )
}

fn set_headers<'a>(
    path: &str,
    params: &str,
    request_headers: &HashMap<String, String>,
    file: &'a AssetFile,
    size: usize,
    code: &mut u16,
    headers: &mut HashMap<&'a str, Cow<'a, str>>,
) -> (usize, usize, Option<StreamingStrategy>) {
    // let mut gzip = false;
    // let mut content_type = "";
    // for (key, value) in file.headers.iter() {
    //     if &key.to_lowercase() == "content-type" {
    //         content_type = value;
    //     }
    //     if &key.to_lowercase() == "content-encoding" && value == "gzip" {
    //         gzip = true;
    //     }
    // }

    // 文件名下载
    if let Ok(reg) = Regex::new(r"attachment=(.*\..*)?(&.*)?$") {
        for cap in reg.captures_iter(params) {
            let mut file_name = cap
                .get(1)
                .map(|m| &params[m.start()..m.end()])
                .unwrap_or("");
            if file_name.is_empty() {
                let s = file.path.split('/');
                for name in s {
                    file_name = name;
                }
            }
            if !file_name.is_empty() {
                headers.insert(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", file_name).into(),
                ); // 下载文件名
            }
        }
    }

    // ! 这个时间库无法编译
    // use chrono::{TimeZone, Utc};
    // let modified = Utc.timestamp_nanos(file.modified as i64);
    // headers.insert("Last-Modified", modified.to_rfc2822().into());

    // 额外增加的请求头
    headers.insert("Accept-Ranges", "bytes".into()); // 支持范围请求
    headers.insert("ETag", file.hash.hex().into()); // 缓存标识

    // 访问控制
    headers.insert("Access-Control-Allow-Origin", "*".into());
    headers.insert(
        "Access-Control-Allow-Methods",
        "HEAD, GET, POST, OPTIONS".into(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Origin,Access-Control-Request-Headers,Access-Control-Allow-Headers,DNT,X-Requested-With,X-Mx-ReqToken,Keep-Alive,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Accept,Connection,Cook ie,X-XSRF-TOKEN,X-CSRF-TOKEN,Authorization".into(),
    );
    headers.insert(
        "Access-Control-Expose-Headers",
        "Accept-Ranges,Content-Length,Content-Range,Transfer-Encoding,Connection,Cache-Control,Content-Disposition"
            .into(),
    );
    headers.insert("Access-Control-Max-Age", "86400".into());

    // Range 设置
    let mut offset: usize = 0;
    let mut offset_end: usize = size;
    if let Some(range) = {
        let mut range = None;
        for (key, value) in request_headers.iter() {
            if &key.to_lowercase() == "range" {
                range = Some(value.trim());
                break;
            }
        }
        range
    } {
        // bytes=start-end
        if let Some(range) = range.strip_prefix("bytes=") {
            let mut ranges = range.split('-');
            let s = ranges.next();
            let e = ranges.next();
            if let Some(s) = s {
                let s: usize = s.parse().unwrap_or(0);
                if s < size {
                    offset = s
                };
            }
            if let Some(e) = e {
                let e: usize = e.parse().unwrap_or(size - 1);
                if offset < e && e < size {
                    offset_end = e + 1
                };
            }
        }
    }

    // 独立的请求头内容
    for (key, value) in file.headers.iter() {
        headers.insert(key, value.into());
    }
    // ic_cdk::println!("---------- {} {} ----------", start, end);
    // 如果过长, 需要阶段显示
    let mut streaming_end = offset_end;
    let mut streaming_strategy: Option<StreamingStrategy> = None;
    let range = streaming_end - offset;
    if MAX_RESPONSE_LENGTH < range {
        // 响应的范围太大了, 缩短为最大长度, 此时应当开启流式响应
        streaming_end = offset + MAX_RESPONSE_LENGTH;
        streaming_strategy = Some(StreamingStrategy::Callback {
            callback: HttpRequestStreamingCallback::new(ic_cdk::id(), "http_streaming".to_string()),
            token: StreamingCallbackToken {
                path: path.to_string(),
                params: params.to_string(),
                headers: request_headers.clone(),
                start: streaming_end as u64,
                end: offset_end as u64,
            },
        });
        headers.insert("Transfer-Encoding", "chunked".into());
        headers.insert("Connection", "keep-alive".into()); // 保持链接
    }
    // Content-Range: bytes 0-499/10000
    headers.insert(
        "Content-Range",
        format!("bytes {}-{}/{}", offset, offset_end - 1, size).into(), // 流式响应也要设置正确的内容范围
    );
    // ! 长度设置了会出错
    // headers.insert("Content-Length", format!("{}", end - start).into()); // ? 这个应该是本次返回的长度

    // 如果是视频可能需要返回其他的
    *code = 200;
    if offset_end < size {
        *code = 206; // 还有内容没给
    }

    (offset, streaming_end - offset, streaming_strategy)
}

// 找不到对应的文件
fn not_found<'a>(code: &mut u16, headers: &mut HashMap<&'a str, Cow<'a, str>>) -> Vec<u8> {
    *code = 404;

    headers.insert("Content-Type", "text/plain".into());

    b"Not found"[..].into()
}

// 流式响应回调
#[ic_cdk::query]
fn http_streaming(
    StreamingCallbackToken {
        path,
        params,
        headers,
        start,
        end,
    }: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    // ic_cdk::println!(
    //     "http_streaming: {:?} {:?} {:?} {:?} {:?}",
    //     path,
    //     params,
    //     headers,
    //     start,
    //     end,
    // );
    if start == end {
        // 首尾相等, 说明没有数据了
        return StreamingCallbackHttpResponse {
            body: vec![],
            token: None,
        };
    }
    crate::stable::with_state(|state| {
        let file = state.business_assets_get_file(&path);
        if let Some(file) = file {
            let asset = state.business_assets_get(&file.hash);
            if let Some(asset) = asset {
                // 如果过长, 需要阶段显示
                let offset = start as usize;
                let offset_end = end as usize;
                let mut streaming_end = offset_end;
                let range = streaming_end - offset;
                if MAX_RESPONSE_LENGTH < range {
                    // 响应的范围太大了, 缩短为最大长度, 此时应当继续流式响应
                    streaming_end = offset + MAX_RESPONSE_LENGTH;
                }
                return StreamingCallbackHttpResponse {
                    body: asset
                        .slice(&file.hash, file.size, offset, streaming_end - offset)
                        .to_vec(),
                    token: Some(StreamingCallbackToken {
                        path: path.to_string(),
                        params: params.to_string(),
                        headers: headers.clone(),
                        start: streaming_end as u64, // 继续
                        end,
                    }),
                };
            }
        }
        StreamingCallbackHttpResponse {
            body: vec![],
            token: None,
        }
    })
}
