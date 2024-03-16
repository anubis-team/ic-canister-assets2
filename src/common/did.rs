// =================== rust 必须自己黑自己获得 did 内容 ===================

/// 暴露出方法, 用这种 mock 方法来告诉 cdk 要生成对应的 did 接口
/// 由于测试方法和真正的方法有冲突，这里和下面的方法进行分开
#[candid::candid_method(query)]
#[cfg(test)]
fn __get_candid_interface_tmp_hack() -> String {
    todo!()
}

/// 这里是具体代码执行的逻辑，非测试编译才包含
/// 一旦有这个，后面测试的方法就不管用了，因此配置非测试环境下包含该方法
#[ic_cdk::query]
#[cfg(not(test))]
fn __get_candid_interface_tmp_hack() -> String {
    #[allow(unused_imports)]
    use crate::types::*;

    candid::export_service!();
    __export_service()
}

// =========== 打印 did ===========  要放到最下面

/// 测试方法 打印输出 candid 文件内容
/// 执行代码
/// `cargo test update_candid -- --nocapture`
#[test]
fn update_candid() {
    #[allow(unused_imports)]
    use crate::types::*;

    candid::export_service!(); // 这一步应该是注入 __export_service 方法

    let text = __export_service(); // 取得 candid 内容

    // std::println!("{}", text); // 控制台输出

    // 输出到对应的文件
    use std::io::Write;
    let filename = "sources/source.did"; // 输出至 did 文件
    let _ = std::fs::remove_file(filename);
    std::fs::File::create(&filename)
        .expect("create failed")
        .write_all(text.as_bytes())
        .expect("write candid failed");
}
