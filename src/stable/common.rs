use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use ic_canister_kit::identity::caller;
use ic_canister_kit::types::*;

use super::ParsePermission;
use super::{schedule_task, CanisterInitialArg, ParsePermissionError, RecordTopics};
use super::{State, State::*};

// 默认值
impl Default for State {
    fn default() -> Self {
        // ? 初始化和升级会先进行迁移, 因此最初的版本无关紧要
        V0(Box::default())
    }
}

/// 检查是否拥有某权限
pub fn check_permission(
    permission: &str,
    running: bool, // 是否要求必须处于正常运行状态
) -> Result<(), String> {
    let caller = ic_canister_kit::identity::caller();
    with_state(|s| {
        if s.permission_has(&caller, &{
            #[allow(clippy::unwrap_used)] // ? SAFETY
            s.parse_permission(permission).unwrap()
        }) {
            if running {
                #[allow(clippy::unwrap_used)] // ? SAFETY
                s.pause_must_be_running().unwrap();
            }
            return Ok(());
        }
        Err(format!("Permission '{}' is required", permission))
    })
}

// ================= 需要持久化的数据 ================

thread_local! {
    static STATE: RefCell<State> = RefCell::default();// 存储系统数据
}

// ==================== 初始化方法 ====================

#[ic_cdk::init]
fn initial(arg: Option<CanisterInitialArg>) {
    with_mut_state_without_record(|s| {
        let record_id = s.record_push(
            caller(),
            RecordTopics::Initial.topic(),
            format!("Initial by {}", caller().to_text()),
        );
        s.upgrade();
        s.init(arg.unwrap_or_else(CanisterInitialArg::none));
        s.schedule_reload(); // * 重置定时任务
        s.record_update(record_id, format!("Version: {}", s.version()));
    })
}

// ==================== 升级时的恢复逻辑 ====================

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    STATE.with(|state| {
        #[allow(clippy::unwrap_used)] // ? SAFETY
        let (record_id, version, bytes): (Option<RecordId>, u32, Vec<u8>) =
            ic_cdk::storage::stable_restore().unwrap();

        // 利用版本号恢复升级前的版本
        let mut last_state = State::from_version(version);
        last_state.heap_from_bytes(&bytes); // 恢复数据
        *state.borrow_mut() = last_state;

        state.borrow_mut().upgrade(); // ! 恢复后要进行升级到最新版本
        let schedule = state.borrow().schedule_find();
        state.borrow_mut().init(CanisterInitialArg { schedule }); // ! 升级到最新版本后, 需要执行初始化操作
        state.borrow_mut().schedule_reload(); // * 重置定时任务
        let version = state.borrow().version();
        if let Some(record_id) = record_id {
            state
                .borrow_mut()
                .record_update(record_id, format!("Next version: {}", version));
        }
    });
}

// ==================== 升级时的保存逻辑，下次升级执行 ====================

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    let caller = caller();
    STATE.with(|state| {
        #[allow(clippy::unwrap_used)] // ? SAFETY
        state.borrow().pause_must_be_paused().unwrap(); // ! 必须是维护状态, 才可以升级
        state.borrow_mut().schedule_stop(); // * 停止定时任务

        let record_id = state.borrow_mut().record_push(
            caller,
            RecordTopics::Upgrade.topic(),
            format!("Upgrade by {}", caller.to_text()),
        );
        let version = state.borrow().version();
        let bytes = state.borrow().heap_to_bytes();
        #[allow(clippy::unwrap_used)] // ? SAFETY
        ic_cdk::storage::stable_save((record_id, version, bytes)).unwrap();
    });
}

// ==================== 工具方法 ====================

/// 外界需要系统状态时
#[allow(unused)]
pub fn with_state<F, R>(callback: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|state| {
        let state = state.borrow(); // 取得不可变对象
        callback(&state)
    })
}

/// 需要可变系统状态时
#[allow(unused)]
pub fn with_mut_state_without_record<F, R>(callback: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        callback(&mut state)
    })
}

/// 需要可变系统状态时 // ! 变更操作一定要记录
#[allow(unused)]
pub fn with_mut_state<F, R>(callback: F, caller: CallerId, topic: RecordTopic, content: String) -> R
where
    F: FnOnce(&mut State) -> (Option<String>, R),
{
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        let record_id = state.record_push(caller, topic, content);
        let (done, result) = callback(&mut state);
        state.record_update(record_id, done.unwrap_or_default());
        result
    })
}

/// 新增记录
#[allow(unused)]
pub fn with_record_push(topic: RecordTopic, content: String) -> RecordId {
    let caller = caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        state.record_push(caller, topic, content)
    })
}
/// 更新记录
#[allow(unused)]
pub fn with_record_update(record_id: RecordId, done: String) {
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        state.record_update(record_id, done)
    })
}
/// 更新记录
#[allow(unused)]
pub fn with_record_update_done(record_id: RecordId) {
    STATE.with(|state| {
        let mut state = state.borrow_mut(); // 取得可变对象
        state.record_update(record_id, String::new())
    })
}

impl Pausable<PauseReason> for State {
    // 查询
    fn pause_query(&self) -> &Option<PauseReason> {
        self.get().pause_query()
    }
    // 修改
    fn pause_replace(&mut self, reason: Option<PauseReason>) {
        self.get_mut().pause_replace(reason)
    }
}

impl ParsePermission for State {
    fn parse_permission<'a>(&self, name: &'a str) -> Result<Permission, ParsePermissionError<'a>> {
        self.get().parse_permission(name)
    }
}

impl Permissable<Permission> for State {
    // 查询
    fn permission_users(&self) -> HashSet<&UserId> {
        self.get().permission_users()
    }
    fn permission_roles(&self) -> HashSet<&String> {
        self.get().permission_roles()
    }
    fn permission_assigned(&self, user_id: &UserId) -> Option<&HashSet<Permission>> {
        self.get().permission_assigned(user_id)
    }
    fn permission_role_assigned(&self, role: &str) -> Option<&HashSet<Permission>> {
        self.get().permission_role_assigned(role)
    }
    fn permission_user_roles(&self, user_id: &UserId) -> Option<&HashSet<String>> {
        self.get().permission_user_roles(user_id)
    }
    fn permission_has(&self, user_id: &UserId, permission: &Permission) -> bool {
        self.get().permission_has(user_id, permission)
    }
    fn permission_owned(&self, user_id: &UserId) -> HashMap<&Permission, bool> {
        self.get().permission_owned(user_id)
    }

    // 修改
    fn permission_reset(&mut self, permissions: HashSet<Permission>) {
        self.get_mut().permission_reset(permissions)
    }
    fn permission_update(
        &mut self,
        args: Vec<PermissionUpdatedArg<Permission>>,
    ) -> Result<(), PermissionUpdatedError<Permission>> {
        self.get_mut().permission_update(args)
    }
}

impl Recordable<Record, RecordTopic, RecordSearch> for State {
    // 查询
    fn record_find_all(&self) -> &[Record] {
        self.get().record_find_all()
    }

    // 修改
    fn record_push(&mut self, caller: CallerId, topic: RecordTopic, content: String) -> RecordId {
        self.get_mut().record_push(caller, topic, content)
    }
    fn record_update(&mut self, record_id: RecordId, done: String) {
        self.get_mut().record_update(record_id, done)
    }

    // 迁移
    fn record_migrate(&mut self, max: u32) -> MigratedRecords<Record> {
        self.get_mut().record_migrate(max)
    }
}

impl Schedulable for State {
    // 查询
    fn schedule_find(&self) -> Option<DurationNanos> {
        self.get().schedule_find()
    }
    // 修改
    fn schedule_replace(&mut self, schedule: Option<DurationNanos>) {
        self.get_mut().schedule_replace(schedule)
    }
}

#[allow(unused)]
fn static_schedule_task() {
    if with_state(|s| s.pause_is_paused()) {
        return; // 维护中不允许执行任务
    }

    ic_cdk::spawn(async move { schedule_task(None).await });
}

pub trait ScheduleTask: Schedulable {
    fn schedule_stop(&self) {
        ic_canister_kit::functions::schedule::stop_schedule();
    }
    fn schedule_reload(&mut self) {
        let schedule = self.schedule_find();
        ic_canister_kit::functions::schedule::start_schedule(&schedule, static_schedule_task);
    }
}

impl ScheduleTask for State {}

impl StableHeap for State {
    fn heap_to_bytes(&self) -> Vec<u8> {
        self.get().heap_to_bytes()
    }

    fn heap_from_bytes(&mut self, bytes: &[u8]) {
        self.get_mut().heap_from_bytes(bytes)
    }
}
