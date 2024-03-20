use ic_canister_kit::{common::option::display_option, identity::caller};

use crate::stable::*;
use crate::types::*;

// ================== 通用接口 ==================

#[ic_cdk::query]
pub fn wallet_balance() -> candid::Nat {
    ic_canister_kit::canister::cycles::wallet_balance()
}

#[ic_cdk::update]
pub fn wallet_receive() -> candid::Nat {
    let caller = caller();
    ic_canister_kit::canister::cycles::wallet_receive(|accepted| {
        let record_id = with_record_push(
            RecordTopics::CyclesCharge.topic(),
            format!("{} recharge {} cycles", caller.to_text(), accepted),
        );
        with_record_update_done(record_id);
    })
}

#[ic_cdk::update]
async fn canister_status() -> ic_cdk::api::management_canister::main::CanisterStatusResponse {
    #[allow(clippy::unwrap_used)]
    ic_canister_kit::canister::status::canister_status(ic_canister_kit::identity::self_canister_id()).await.unwrap()
}

#[ic_cdk::query]
async fn whoami() -> ic_canister_kit::types::UserId {
    ic_canister_kit::identity::caller()
}

// ================== 数据版本 ==================

// 当前数据库版本
#[ic_cdk::query]
fn version() -> u32 {
    with_state(|s| s.version())
}

// ================== 维护接口 ==================

// 查询维护状态
#[ic_cdk::query(guard = "has_pause_query")]
fn pause_query() -> bool {
    with_state(|s| s.pause_is_paused())
}

// 查询维护状态
#[ic_cdk::query(guard = "has_pause_query")]
fn pause_query_reason() -> Option<PauseReason> {
    with_state(|s| s.pause_query().clone())
}

// 设置维护状态
#[ic_cdk::update(guard = "has_pause_replace")]
fn pause_replace(reason: Option<String>) {
    let old = with_state(|s| s.pause_query().clone());

    if old.is_none() && reason.is_none() {
        return; // 未改变内容
    }

    let caller = caller();
    let arg_content = format!("{} -> {}", display_option(&old), display_option(&reason)); // * 记录参数内容

    with_mut_state(
        |s| {
            s.pause_replace(reason.map(PauseReason::new));
            (None, ())
        },
        caller,
        RecordTopics::Pause.topic(),
        arg_content,
    )
}

// ================== 权限接口 ==================

// 所有权限
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_all() -> Vec<Permission> {
    with_state(|s| {
        ACTIONS
            .into_iter()
            .map(|name| {
                #[allow(clippy::unwrap_used)] // ? SAFETY
                s.parse_permission(name).unwrap()
            })
            .collect()
    })
}

// 查询自己权限
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_query() -> Vec<&'static str> {
    permission_find_by_user(ic_canister_kit::identity::caller())
}

// 查询他人权限
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_find_by_user(user_id: UserId) -> Vec<&'static str> {
    with_state(|s| {
        ACTIONS
            .into_iter()
            .filter(|permission| {
                s.permission_has(&user_id, &{
                    #[allow(clippy::unwrap_used)] // ? SAFETY
                    s.parse_permission(permission).unwrap()
                })
            })
            .collect()
    })
}

// 查询自己指定权限
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_assigned_query() -> Option<HashSet<Permission>> {
    permission_assigned_by_user(ic_canister_kit::identity::caller())
}

// 查询他人指定权限
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_assigned_by_user(user_id: UserId) -> Option<HashSet<Permission>> {
    with_state(|s| s.permission_assigned(&user_id).cloned())
}

// 所有角色
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_roles_all() -> HashMap<String, HashSet<Permission>> {
    with_state(|s| {
        s.permission_roles()
            .into_iter()
            .map(|role| {
                (
                    role.to_owned(),
                    s.permission_role_assigned(role)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect()
    })
}

// 查询自己角色
#[ic_cdk::query(guard = "has_permission_query")]
fn permission_roles_query() -> Option<HashSet<String>> {
    permission_roles_by_user(ic_canister_kit::identity::caller())
}

// 查询他人角色
#[ic_cdk::query(guard = "has_permission_find")]
fn permission_roles_by_user(user_id: UserId) -> Option<HashSet<String>> {
    with_state(|s| s.permission_user_roles(&user_id).cloned())
}

// 更新权限
#[ic_cdk::update(guard = "has_permission_update")]
fn permission_update(args: Vec<PermissionUpdatedArg<String>>) {
    let caller = caller();
    let arg_content = format!(
        "permission update: [{}]",
        args.iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    ); // * 记录参数内容

    with_mut_state(
        |s| {
            #[allow(clippy::unwrap_used)] // ? SAFETY
            let args = args
                .into_iter()
                .map(|a| a.parse_permission(|n| s.parse_permission(n).map_err(|e| e.to_string())))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            #[allow(clippy::unwrap_used)] // ? SAFETY
            s.permission_update(args).unwrap();
            (None, ())
        },
        caller,
        RecordTopics::Permission.topic(),
        arg_content,
    )
}

// ================== 记录接口 ==================

// 查询所有主题
#[ic_cdk::query(guard = "has_record_find")]
fn record_topics() -> Vec<String> {
    RecordTopics::topics()
}

// 查询所有 分页
#[ic_cdk::query(guard = "has_record_find")]
fn record_find_by_page(page: QueryPage, search: Option<RecordSearchArg>) -> PageData<Record> {
    #[allow(clippy::unwrap_used)] // ? SAFETY
    let search = search
        .map(|s| s.into(|t| RecordTopics::from(t).map(|t| t.topic())))
        .transpose()
        .unwrap();
    #[allow(clippy::unwrap_used)] // ? SAFETY
    with_state(|s| {
        s.record_find_by_page(&page, 1000, &search)
            .map(|p| p.into())
    })
    .unwrap()
}

// 移动
#[ic_cdk::update(guard = "has_record_migrate")]
fn record_migrate(max: u32) -> MigratedRecords<Record> {
    let caller = caller();
    let arg_content = format!("wanna migrate {} records", max); // * 记录参数内容

    with_mut_state(
        |s| {
            let result = s.record_migrate(max);

            (
                Some(format!(
                    "removed: {} total: {} record size: {} ",
                    result.removed,
                    result.next_id,
                    result.records.len(),
                )),
                result,
            )
        },
        caller,
        RecordTopics::Schedule.topic(),
        arg_content,
    )
}

// ================== 定时任务 ==================

// 查询定时状态
#[ic_cdk::query(guard = "has_schedule_find")]
fn schedule_find() -> Option<u64> {
    with_state(|s| s.schedule_find().map(|s| s.into_inner() as u64))
}

// 设置定时状态
#[ic_cdk::update(guard = "has_schedule_replace")]
fn schedule_replace(schedule: Option<u64>) {
    let old = with_state(|s| s.schedule_find());

    let caller = caller();
    let arg_content = format!("{} -> {}", display_option(&old), display_option(&schedule)); // * 记录参数内容

    with_mut_state(
        |s| {
            s.schedule_replace(schedule.map(|s| (s as u128).into()));
            s.schedule_reload(); // * 重置定时任务
            (None, ())
        },
        caller,
        RecordTopics::Schedule.topic(),
        arg_content,
    )
}

// 手动触发定时任务
#[ic_cdk::update(guard = "has_schedule_trigger")]
async fn schedule_trigger() {
    let caller = caller();

    #[allow(clippy::unwrap_used)] // ? SAFETY
    with_mut_state_without_record(|s| {
        s.pause_must_be_running() // 维护中不允许执行任务
    })
    .unwrap();

    schedule_task(Some(caller)).await;
}
