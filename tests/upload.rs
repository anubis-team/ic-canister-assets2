/// 上传文件
/// cargo test upload
use std::collections::HashMap;

#[derive(Debug)]
struct LocalFile {
    pub path: String,
    pub size: u64,
    pub headers: Vec<(String, String)>,
    pub created: u64,
    pub hash: String,
    pub data: Vec<u8>,
}
#[derive(Debug)]
struct RemoteFile {
    pub path: String,
    pub size: u64,
    pub headers: Vec<(String, String)>,
    pub created: u64,
    pub hash: String,
}

struct UploadFile {
    pub file: LocalFile,
    pub chunks: u32,     //  总块数
    pub chunk_size: u64, // 块大小
    pub index: u64,      // 序号
    pub offset: u64,     // 起始偏移
    pub offset_end: u64, // 末位
}

fn get_content_type_with_gz(name: String, ext: String, gz: bool) -> String {
    let mut content_type = "";
    match ext.as_str() {
        // 文本
        "txt" => content_type = "text/plain",
        "html" | "htm" | "htx" | "xhtml" => content_type = "text/html",
        "css" => content_type = "text/css",
        "js" => content_type = "text/javascript",
        "md" => content_type = "text/markdown",
        "ics" => content_type = "text/calendar",
        "csv" => content_type = "text/csv",
        "xml" => content_type = "text/xml",
        // 应用
        // "js" => content_type = "application/javascript",
        "json" => content_type = "application/json",
        // "xml" => content_type = "application/xml",
        "pdf" => content_type = "application/pdf",
        "zip" => content_type = "application/zip",
        "7z" => content_type = "application/x-7z-compressed",
        // 图片
        "png" => content_type = "image/png",
        "gif" => content_type = "image/gif",
        "jpg" | "jpeg" => content_type = "image/jpeg",
        "bmp" => content_type = "image/bmp",
        "svg" => content_type = "image/svg+xml",
        "webp" => content_type = "image/webp",
        "tif" | "tiff" => content_type = "image/tiff",
        "ico" => content_type = "image/x-icon",
        // 视频
        "mp4" => content_type = "video/mp4",
        "avi" => content_type = "video/x-msvideo",
        "mov" => content_type = "video/quicktime",
        "mpeg" => content_type = "video/mpeg",
        "ogv" => content_type = "video/ogg",
        "webm" => content_type = "video/webm",
        // 音频
        "mp3" => content_type = "audio/mp3",
        "wav" => content_type = "audio/wav",
        // "ogg" => content_type = "audio/ogg",
        "flac" => content_type = "audio/flac",
        "aac" => content_type = "audio/aac",
        "weba" => content_type = "audio/webm",
        "oga" => content_type = "audio/ogg",
        "wma" => content_type = "audio/x-ms-wma",
        "mid" | "midi" => content_type = "audio/midi",
        "ra" | "ram" => content_type = "audio/x-realaudio",
        // 字体
        "otf" => content_type = "font/otf",
        "ttf" => content_type = "font/ttf",
        "woff" => content_type = "font/woff",
        "woff2" => content_type = "font/woff2",
        // 其他
        "dat" => {}
        "plot" => {}
        "cache" => {}
        "gz" => {
            if gz {
                let mut ext = "";
                let mut s = (&name[0..(name.len() - 3)]).split(".");
                while let Some(e) = s.next() {
                    ext = e;
                }
                return get_content_type_with_gz(name.to_string(), ext.to_string(), false);
            } else {
                panic!("Unknown file type: {}", ext.to_lowercase().as_str())
            }
        }
        _ => panic!("Unknown file type: {}", ext.to_lowercase().as_str()),
    }
    content_type.to_string()
}
fn get_headers(file: &str) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = vec![];

    let mut content_type: String = String::from("");

    use std::path::Path;
    let file_path = Path::new(file);
    if let Some(extension) = file_path.extension() {
        if let Some(ext_str) = extension.to_str() {
            content_type = get_content_type_with_gz(file.to_string(), ext_str.to_lowercase(), true);
        } else {
            println!("Invalid extension");
        }
    } else {
        println!("No extension");
    }

    // 内容类型
    if !content_type.is_empty() {
        headers.push(("Content-Type".to_string(), content_type.to_string()));
    }

    // 缓存时间
    headers.push((
        "Cache-Control".to_string(),
        "public, max-age=31536000".to_string(),
    ));

    // gzip
    if file.ends_with(".gz") {
        headers.push(("Content-Encoding".to_string(), "gzip".to_string()));
    }

    headers
}

#[test]
fn upload() {
    // 0. 调用身份
    let identity = "default";
    // 1. 读取本地数据
    let mut local_files: HashMap<String, LocalFile> = HashMap::new();
    load_local_files("assets", &mut local_files);
    let local_file_names: Vec<String> = local_files.iter().map(|(_, f)| f.path.clone()).collect();
    for (_, file) in local_files.iter() {
        println!("{} -> {}", file.path, file.size);
    }
    // 2. 读取线上数据
    let remote_files = load_remote_files(identity);
    println!("remote files: {:?}", remote_files);
    // 3. 比较远程有但是本地没有的要删除
    let deletes: Vec<String> = remote_files
        .iter()
        .map(|f| f.path.clone())
        .filter(|p| !local_file_names.contains(p)) // 远程存在, 但本地不存在
        .collect();
    delete_files(identity, deletes);
    // 4. 比较本地有但是远程不一样的要进行上传
    let local_files: Vec<LocalFile> = local_files
        .into_iter()
        .map(|(_, f)| f)
        .filter(|file| {
            let remote_file = remote_files.iter().find(|f| f.path == file.path);
            if remote_file.is_none() {
                return true; // 本地有, 远程没有
            }
            let remote_file = remote_file.unwrap();
            // 有文件就比较一下其他信息是否一致
            let mut file_headers: Vec<String> = file
                .headers
                .iter()
                .map(|h| format!("{}:{}", h.0, h.1))
                .collect();
            file_headers.sort();
            let mut remote_file_headers: Vec<String> = remote_file
                .headers
                .iter()
                .map(|h| format!("{}:{}", h.0, h.1))
                .collect();
            remote_file_headers.sort();
            return file.size != remote_file.size
                || file_headers.join(";") != remote_file_headers.join(";")
                || file.hash != remote_file.hash;
        })
        .collect();
    upload_files(identity, local_files);
}

// =========== 上传文件 ===========

fn upload_files(identity: &str, local_files: Vec<LocalFile>) {}
fn upload_file(identity: &str, local_files: Vec<UploadFile>, index: usize) {}

// =========== 删除文件 ===========

fn delete_files(identity: &str, names: Vec<String>) {
    use std::process::Command;

    let _start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    let args = format!(
        "(vec {{{}}})",
        names
            .into_iter()
            .map(|name| format!("\"{}\"", name))
            .collect::<Vec<String>>()
            .join(";")
    );

    // let output = Command::new("pwd")
    //     .current_dir(".")
    //     .output()
    //     .expect("error");
    let output = Command::new("/usr/local/bin/dfx")
        .current_dir(".")
        .arg("--identity")
        .arg(identity)
        .arg("canister")
        .arg("--network")
        .arg("ic")
        .arg("call")
        .arg("ic-canister-assets")
        .arg("delete")
        .arg(&args)
        .arg("--output")
        .arg("idl")
        .output()
        .expect("error");

    let _end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    // println!("api: {} -> {:?}", "files", _end - _start);
    // println!("status: {}", output.status);

    if format!("{}", output.status).eq("exit status: 0") {
        let output = String::from_utf8(output.stdout.clone()).unwrap();
        // println!("output: {}", output);
        return;
    }

    // if format!("{}", output.status) != String::from("0") && result.is_err() {
    //     let output = String::from_utf8(output.stderr.clone()).unwrap();
    //     let words = result.err().unwrap();
    //     if trim(&output).contains(&trim(words)) {
    //         return String::new();
    //     }
    // }

    eprintln!(">>>>>>>>>> ERROR <<<<<<<<<<<");
    eprintln!("identity: {}", identity);
    eprintln!("api: {}", "delete");
    eprintln!("arg: {}", args);
    eprintln!("status: {}", output.status);
    if format!("{}", output.status).eq("exit status: 0") {
        eprintln!(
            "output: {}",
            String::from_utf8(output.stdout).unwrap().trim_end()
        );
    } else {
        eprintln!(
            "error : {}",
            String::from_utf8(output.stderr).unwrap().trim_end()
        );
    }
    panic!("error");
}

// =========== 读取远程文件 ===========

fn load_remote_files(identity: &str) -> Vec<RemoteFile> {
    use std::process::Command;

    let _start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    // let output = Command::new("pwd")
    //     .current_dir(".")
    //     .output()
    //     .expect("error");
    let output = Command::new("/usr/local/bin/dfx")
        .current_dir(".")
        .arg("--identity")
        .arg(identity)
        .arg("canister")
        .arg("--network")
        .arg("ic")
        .arg("call")
        .arg("ic-canister-assets")
        .arg("files")
        .arg("()")
        .arg("--output")
        .arg("idl")
        .output()
        .expect("error");

    let _end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");

    // println!("api: {} -> {:?}", "files", _end - _start);
    // println!("status: {}", output.status);

    if format!("{}", output.status).eq("exit status: 0") {
        let output = String::from_utf8(output.stdout.clone()).unwrap();
        // println!("output: {}", output);
        return parse_remote_files(output);
    }

    // if format!("{}", output.status) != String::from("0") && result.is_err() {
    //     let output = String::from_utf8(output.stderr.clone()).unwrap();
    //     let words = result.err().unwrap();
    //     if trim(&output).contains(&trim(words)) {
    //         return String::new();
    //     }
    // }

    eprintln!(">>>>>>>>>> ERROR <<<<<<<<<<<");
    eprintln!("identity: {}", identity);
    eprintln!("api: {}", "files");
    eprintln!("arg: {}", "");
    eprintln!("status: {}", output.status);
    if format!("{}", output.status).eq("exit status: 0") {
        eprintln!(
            "output: {}",
            String::from_utf8(output.stdout).unwrap().trim_end()
        );
    } else {
        eprintln!(
            "error : {}",
            String::from_utf8(output.stderr).unwrap().trim_end()
        );
    }
    panic!("error");
}

fn parse_remote_files(output: String) -> Vec<RemoteFile> {
    let output = output.trim();
    // println!("output: {} {}", output.len(), output);
    // let output = String::from("(vec {})");
    // let output = String::from(
    //     r#"(vec { record { created = 1_692_724_516_026_887_821 : nat64; hash = "a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222"; path = "/123"; size = 2 : nat64; headers = vec { record { "Content-Type"; "images/png";};};};})"#,
    // );
    // let output = String::from(
    //     r#"(vec { record { created = 1_692_724_516_026_887_821 : nat64; hash = "a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222"; path = "/123"; size = 2 : nat64; headers = vec { record { "Content-Type"; "images/png";};};}; record { created = 1_692_724_566_030_935_897 : nat64; hash = "a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222"; path = "/1234"; size = 2 : nat64; headers = vec { record { "Content-Type"; "images/png";};};}; record { created = 1_692_726_478_497_092_449 : nat64; hash = "a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222"; path = "/123456"; size = 2 : nat64; headers = vec {};}; record { created = 1_692_726_276_101_996_115 : nat64; hash = "a12871fee210fb8619291eaea194581cbd2531e4b23759d225f6806923f63222"; path = "/12345"; size = 2 : nat64; headers = vec { record { "Content-Type"; "images/png";}; record { "ddddd"; "xxxx";};};};})"#,
    // );
    let output = (&output[6..(output.len() - 2)]).to_string();
    let output = output.trim();

    // println!("output1: {} {}", output.len(), output);

    if output.len() == 0 {
        return vec![];
    }

    let output = (&output[9..(output.len() - 4)]).to_string();
    let output = output.trim();
    // println!("output2: {} {}", output.len(), output);

    let mut files = vec![];
    let mut splitted = output.split("};}; record { ");
    while let Some(content) = splitted.next() {
        // println!("content: {} {}", content.len(), content);
        let content = (&content[10..]).to_string();
        let created: u64 = content
            .split(r#" : nat64; hash = ""#)
            .next()
            .unwrap()
            .to_string()
            .replace("_", "")
            .parse()
            .unwrap();
        // println!("created: {}", created);
        let mut content = content.split(r#" : nat64; hash = ""#);
        content.next();
        let content = content.next().unwrap();
        let hash = (&content[0..64]).to_string();
        // println!("hash: {}", hash);
        let mut content = content.split(r#""; path = ""#);
        content.next();
        let content = content.next().unwrap();
        let path = content.split(r#""; size = "#).next().unwrap().to_string();
        // println!("path: {}", path);
        let mut content = content.split(r#""; size = "#);
        content.next();
        let content = content.next().unwrap();
        let size: u64 = content
            .split(r#" : nat64; headers = "#)
            .next()
            .unwrap()
            .to_string()
            .replace("_", "")
            .parse()
            .unwrap();
        // println!("size: {}", size);
        let mut content = content.split(r#" : nat64; headers = "#);
        content.next();
        let content = content.next().unwrap();
        let headers: Vec<(String, String)> = if 5 < content.len() {
            let content = &content[16..(content.len() - 4)];
            let mut headers = vec![];
            let mut cs = content.split(r#"";}; record { ""#);
            while let Some(s) = cs.next() {
                let mut ss = s.split(r#""; ""#);
                let key = ss.next().unwrap().to_string();
                let value = ss.next().unwrap().to_string();
                headers.push((key, value));
            }
            headers
        } else {
            vec![]
        };
        // println!("headers: {:?}", headers);
        files.push(RemoteFile {
            path,
            size,
            headers,
            created,
            hash,
        });
    }
    // println!("remote files: {:?}", files);
    files
}

// =========== 读取本地文件 ===========

fn load_local_files(dir_path: &str, files: &mut HashMap<String, LocalFile>) {
    let entries = std::fs::read_dir(dir_path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let file_type = entry.file_type().unwrap();

        if file_type.is_file() {
            let path = format!("{}/{}", dir_path, file_name.to_str().unwrap().to_string());
            let file = load_local_file(&path);
            files.insert(path, file);
        } else if file_type.is_dir() {
            let path = format!("{}/{}", dir_path, file_name.to_str().unwrap().to_string());
            load_local_files(&path, files);
        }
    }
}

fn load_local_file(path: &str) -> LocalFile {
    // 获取文件大小
    let metadata = std::fs::metadata(path).unwrap();
    let file_size = metadata.len();

    // 读取文件内容
    let mut file = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut buffer).unwrap();

    LocalFile {
        path: path.to_string(),
        size: file_size,
        headers: get_headers(&path),
        created: 0,
        hash: do_hash(&buffer),
        data: buffer,
    }
}

fn do_hash(data: &Vec<u8>) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&data[..]);
    let digest: [u8; 32] = hasher.finalize().into();
    hex::encode(&digest)
}
