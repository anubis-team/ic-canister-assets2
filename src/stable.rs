use std::cell::{RefCell, RefMut};

use ic_canister_kit::{
    identity::caller,
    types::{Initial, Permissions, PermissionsState, Stable},
};

use crate::core::types::{CoreAssets, CoreAssetsState, UploadingAssets, UploadingAssetsState};

pub const PERMISSION_ADMIN: &str = "admin"; // 所有权限

#[derive(Debug, Default)]
pub struct State {
    pub permissions: Permissions,
    pub assets: CoreAssets,
    pub uploading: UploadingAssets,
}

impl Initial for State {
    fn init(&mut self) {
        self.permissions.insert(PERMISSION_ADMIN, caller());
    }
}

type RestoreState = (PermissionsState, CoreAssetsState, UploadingAssetsState);
type SaveState = RestoreState;

impl Stable<SaveState, RestoreState> for State {
    fn store(&mut self) -> SaveState {
        (
            self.permissions.store(),
            self.assets.store(),
            self.uploading.store(),
        )
    }

    fn restore(&mut self, restore: RestoreState) {
        self.permissions.restore(restore.0);
        self.assets.restore(restore.1);
        self.uploading.restore(restore.2);
    }
}

// ================= 需要持久化的数据 ================

thread_local! {
    // 存储系统数据
    static STATE: RefCell<State> = RefCell::default();
}

// ==================== 升级时的恢复逻辑 ====================

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    STATE.with(|state_ref| {
        let mut state: RefMut<dyn Stable<SaveState, RestoreState>> = state_ref.borrow_mut();
        ic_canister_kit::stable::post_upgrade(&mut state);
    });
}

// ==================== 升级时的保存逻辑，下次升级执行 ====================

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    STATE.with(|state_ref| {
        let mut state: RefMut<dyn Stable<SaveState, RestoreState>> = state_ref.borrow_mut();
        ic_canister_kit::stable::pre_upgrade(&mut state);
    });
}

// 工具方法

/// 外界需要系统状态时
pub fn with_state<F, R>(callback: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|_state| {
        let state = _state.borrow(); // 取得不可变对象
        callback(&state)
    })
}

/// 需要可变系统状态时
pub fn with_mut_state<F, R>(callback: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|_state| {
        let mut state = _state.borrow_mut(); // 取得不可变对象
        callback(&mut state)
    })
}

// 相关方法

#[ic_cdk::init]
fn initial() {
    with_mut_state(|s| s.init())
}

pub fn is_admin() -> Result<(), String> {
    let caller = caller();
    with_state(|s| {
        if s.permissions.has_permission(PERMISSION_ADMIN, caller) {
            return Ok(());
        }
        return Err(format!("{} is not admin", caller.to_text()));
    })
}
