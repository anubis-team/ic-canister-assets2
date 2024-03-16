use super::super::v000::types::InnerState as LastState;

use super::types::*;

impl From<Box<LastState>> for Box<InnerState> {
    fn from(value: Box<LastState>) -> Self {
        let mut state = InnerState::default(); // ? 初始化

        // 1. 继承之前的数据
        let LastState {
            pause,
            permissions,
            records,
            schedule,
        } = *value;
        state.pause = pause;
        state.permissions = permissions;
        state.records = records;
        state.schedule = schedule;

        Box::new(state)
    }
}
