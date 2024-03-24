use super::super::v000::types::{CanisterKit as LastCanisterKit, InnerState as LastState};

use super::types::*;

impl From<Box<LastState>> for Box<InnerState> {
    fn from(value: Box<LastState>) -> Self {
        let mut state = InnerState::default(); // ? 初始化

        // ! 每次升级新版本，务必比较每一个数据的升级方式
        // ! 如果不修改数据结构，可以直接赋值升级
        // ! 如果修改数据结构，必须代码处理数据升级

        // 1. 继承之前的数据
        let LastCanisterKit {
            pause,
            permissions,
            records,
            schedule,
        } = value.canister_kit;
        state.canister_kit.pause = pause;
        state.canister_kit.permissions = permissions;
        state.canister_kit.records = records;
        state.canister_kit.schedule = schedule;

        Box::new(state)
    }
}
