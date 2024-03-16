use std::{borrow::Cow, collections::HashMap};

use crate::stable::{Business, State};

pub const HTML: &str = include_str!("../web/index.html");
pub const CSS: &str = include_str!("../web/index.css");

pub fn explore<'a>(headers: &mut HashMap<&'a str, Cow<'a, str>>, state: &State) -> Vec<u8> {
    headers.insert("Content-Type", "text/html".into());

    let files = state.business_files();
    let mut json = String::from("");
    json.push('[');
    json.push_str(
        &files
            .iter()
            .map(|file| {
                format!(
                    "{{path:\"{}\",size:{},headers:[{}],created:{},modified:{},hash:\"{}\"}}",
                    file.path,
                    file.size,
                    file.headers
                        .iter()
                        .map(|(key, value)| format!("{{key:\"{}\",value:\"{}\"}}", key, value))
                        .collect::<Vec<String>>()
                        .join(","),
                    file.created.into_inner() / 1000000,
                    file.modified.into_inner() / 1000000,
                    file.hash
                )
            })
            .collect::<Vec<String>>()
            .join(","),
    );
    json.push(']');

    HTML.replace("/* CSS */", CSS)
        .replace("const _files = [];", &format!("const _files = {};", json))[..]
        .into()
}
