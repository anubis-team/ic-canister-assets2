#!!/bin/bash

cargo clippy

# 部署代码
# dfx deploy --network ic ic-canister-assets --mode=reinstall --yes

# dfx canister --network ic call ic-canister-assets pause_replace "(opt \"for updating\")"
# dfx deploy --network ic ic-canister-assets
# dfx canister --network ic call ic-canister-assets pause_replace "(null)"
# dfx canister --network ic call ic-canister-assets business_hashed_update "(true)"

# dfx deploy --network local ic-canister-assets --mode=reinstall --yes
dfx canister --network local call ic-canister-assets pause_replace "(opt \"for updating\")"
dfx deploy --network local ic-canister-assets
dfx canister --network local call ic-canister-assets pause_replace "(null)"
dfx canister --network local call ic-canister-assets business_hashed_update "(true)"

# 上传资源文件
RUST_BACKTRACE=1 cargo test upload -- --nocapture
