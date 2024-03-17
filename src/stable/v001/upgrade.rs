use super::super::v000::types::{HeapState as LastHeapState, InnerState as LastState};

use super::types::*;

impl From<Box<LastState>> for Box<InnerState> {
    fn from(value: Box<LastState>) -> Self {
        let mut state = InnerState::default(); // ? 初始化

        // 1. 继承之前的数据
        let LastHeapState {
            pause,
            permissions,
            records,
            schedule,
        } = value.heap_state;
        state.heap_state.pause = pause;
        state.heap_state.permissions = permissions;
        state.heap_state.records = records;
        state.heap_state.schedule = schedule;

        Box::new(state)
    }
}
