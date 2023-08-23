use std::{borrow::Cow, collections::HashMap};

use crate::stable::State;

pub fn home<'a>(headers: &mut HashMap<&'a str, Cow<'a, str>>, state: &State) -> Vec<u8> {
    headers.insert("Content-Type", "text/html".into());

    let files = state.assets.files();
    let mut json = String::from("");
    json.push_str("[");
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
                    file.created / 1000000,
                    file.modified / 1000000,
                    file.hash
                )
            })
            .collect::<Vec<String>>()
            .join(","),
    );
    json.push_str("]");

    r#"<!doctype html>
    <html lang="en">
    <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Assets</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.1/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-4bw+/aepP/YC94hEpVNVgiZdgIC5+VKNBQNGCHeKRQN+PtmoHDEXuppvnDJzQIu9" crossorigin="anonymous">
    <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
    </head>
    <body>
    <div id="app">{{ message }}</div>
    <script>
        const { createApp, ref } = Vue
        createApp({
            setup() {
                const message = ref('Hello vue!')
                const files = FILES;
                return {
                    message
                }
            }
        }).mount('#app')
    </script>
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.1/dist/js/bootstrap.bundle.min.js" integrity="sha384-HwwvtgBNo3bZJJLYd8oVXjrBZt8cqVSpeBNS5n7C8IVInixGAoxmnlMuBnhbgrkm" crossorigin="anonymous"></script>
    </body>
    </html>"#.replace("FILES", &json)[..]
        .into()
}
