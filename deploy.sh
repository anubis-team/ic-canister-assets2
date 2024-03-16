#!!/bin/bash

# 更新 candid 文件
cargo test update_candid -- --nocapture

cargo clippy

# 部署代码
# dfx deploy --network ic ic-canister-assets --mode=reinstall --yes
# dfx deploy --network ic ic-canister-assets
dfx deploy --network local ic-canister-assets --mode=reinstall --yes

# 上传资源文件
RUST_BACKTRACE=1 cargo test upload -- --nocapture
