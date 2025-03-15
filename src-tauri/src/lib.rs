// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod executor;
mod openapi;

use crate::executor::Executor;
use crate::openapi::auth::Auth;
use crate::openapi::base::Base;
use crate::openapi::login::{CheckResp, Login};
use crate::openapi::member::{Member};
use crate::openapi::rule::{
    AutoReplyRule, AutoReplyRuleService, Group, GroupService, OpenAi, OpenAiConfig,
    OpenAiConfigService, OpenAiService, ScheduledRule, ScheduledRuleService, APPDATA_PATH,
};
use once_cell::sync::Lazy;
use openapi::client::WxOpenapiClient;
use std::env;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use tauri::path::BaseDirectory;
use tauri::{command, generate_context, Manager, State};
use tokio::sync::{Mutex};

struct AppState {
    uuid: String,
    login: bool,
    base: Base,
    member: Member,
    auth: Auth,
    client: WxOpenapiClient,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            uuid: String::default(),
            login: false,
            base: Base::new(),
            member: Member::new(),
            auth: Auth::new(),
            client: WxOpenapiClient::new(),
        }
    }
}

static WX_CLIENT: Lazy<WxOpenapiClient> = Lazy::new(|| WxOpenapiClient::new());

#[command]
async fn refresh(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    println!("[refresh][main] qr code");

    let mut state = state.lock().await;
    state.client = WxOpenapiClient::new();
    let mut login = Login::new();
    let uuid = login.get_uuid(state.client.client()).await.unwrap();
    state.uuid = uuid.clone();
    let data = login
        .get_qr_code(state.client.client(), &uuid)
        .await
        .unwrap();
    let mut file = File::create(
        APPDATA_PATH
            .lock()
            .unwrap()
            .join("image/code/qr_code.jpg")
            .as_path(),
    )
    .unwrap();
    file.write_all(&data[..]).unwrap();
    file.flush().unwrap();
    Ok(())
}

#[command]
async fn login_check(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    let mut state = state.lock().await;
    println!("login_check");
    if state.login {
        println!("already logged in");
        return Ok("200".to_string());
    }
    let mut login = Login::new();
    let resp = login
        .check(state.client.client(), &state.uuid)
        .await
        .unwrap_or(CheckResp::default());
    if resp.code == "200" {
        state.login = true;
        println!("successfully logged in");
        let base = Base::new()
            .init(state.client.client(), &resp.redirect_uri)
            .await
            .unwrap();
        state.base = base.clone();
    }
    println!("login check code: {}", resp.code);
    Ok(resp.code)
}

#[command]
async fn get_members(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("load wechat members");
    let mut state = state.lock().await;

    state.member = Member::new()
        .init(state.client.client(), &state.base)
        .await
        .unwrap();
    // member_list: Vec<ContactMember>

    let data = serde_json::to_string(&state.member.member_list).unwrap();
    println!("members={:?}", &data);
    Ok(data)
}

#[command]
async fn get_auth(state: State<'_, Mutex<AppState>>) -> Result<i64, ()> {
    println!("load wechat auth");
    let mut state = state.lock().await;

    state.auth = Auth::new().init().await.unwrap();

    Ok(state.auth.expired_time())
}

#[command]
async fn get_user(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("load user");
    let state = state.lock().await;
    let data = serde_json::to_string(&state.base.user).unwrap();
    Ok(data)
}

#[command]
fn create_group(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("create group");
    let group: Group = serde_json::from_str(&config).unwrap();
    let groups = GroupService::new().create(group).unwrap();
    let configs = serde_json::to_string(&groups).unwrap();
    Ok(configs)
}

#[command]
fn get_groups(_: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("get groups");
    let groups = GroupService::new().m_get().unwrap();
    let configs = serde_json::to_string(&groups).unwrap();
    println!("groups={:?}", &groups);
    Ok(configs)
}

#[command]
fn del_group(_: State<'_, Mutex<AppState>>, id: String) -> Result<String, ()> {
    println!("del group");
    let groups = GroupService::new().del(id).unwrap();
    let configs = serde_json::to_string(&groups).unwrap();
    println!("groups={:?}", &groups);
    Ok(configs)
}

#[command]
fn create_auto_reply_rule(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("create auto reply rule");
    let rule: AutoReplyRule = serde_json::from_str(&config).unwrap();
    let rules = AutoReplyRuleService::new().create(rule).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    Ok(configs)
}

#[command]
async fn get_auto_reply_rules(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("get auto reply rules");
    let mut state = state.lock().await;
    let rules = AutoReplyRuleService::new()
        .m_get_with_status_update(&state.auth)
        .unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}

#[command]
fn del_auto_reply_rule(_: State<'_, Mutex<AppState>>, id: String) -> Result<String, ()> {
    println!("del auto reply rule");
    let rules = AutoReplyRuleService::new().del(id).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}

#[command]
fn update_auto_reply_rule(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("update auto reply rule");
    let rule: AutoReplyRule = serde_json::from_str(&config).unwrap();
    let rules = AutoReplyRuleService::new().update(rule).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", configs);
    Ok(configs)
}

#[command]
async fn switch_auto_reply_rule_status(
    state: State<'_, Mutex<AppState>>,
    id: String,
) -> Result<String, ()> {
    println!("switch auto reply rule status");
    let state = state.lock().await;
    let resp = AutoReplyRuleService::new().switch_status(id,&state.auth).await.unwrap();
    println!("switch resp={:?}", &resp);
    Ok(serde_json::to_string(&resp).unwrap())
}

#[command]
fn create_openai(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("create openai");
    let openai: OpenAi = serde_json::from_str(&config).unwrap();
    let rules = OpenAiService::new().create(openai).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    Ok(configs)
}

#[command]
fn get_openai_list(_: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("get openai list");
    let openai_list = OpenAiService::new().m_get().unwrap();
    let configs = serde_json::to_string(&openai_list).unwrap();
    println!("openai_list={:?}", &openai_list);
    Ok(configs)
}

#[command]
fn del_openai(_: State<'_, Mutex<AppState>>, id: String) -> Result<String, ()> {
    println!("del openai");
    let openai_list = OpenAiService::new().del(id).unwrap();
    let configs = serde_json::to_string(&openai_list).unwrap();
    println!("openai_list={:?}", &openai_list);
    Ok(configs)
}

#[command]
fn update_openai(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("update openai");
    let openai: OpenAi = serde_json::from_str(&config).unwrap();
    let openai_list = OpenAiService::new().update(openai).unwrap();
    let configs = serde_json::to_string(&openai_list).unwrap();
    println!("openai_list={:?}", &openai_list);
    Ok(configs)
}

#[command]
async fn test_openai(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("test openai");
    let openai: OpenAi = serde_json::from_str(&config).unwrap();
    let content = OpenAiService::new().test(openai).await.unwrap();
    println!("content={}", &content);
    Ok(content)
}

#[command]
fn get_openai_config(_: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("get openai config");
    let data = OpenAiConfigService::new().m_get().unwrap();
    let configs = serde_json::to_string(&data).unwrap();
    println!("configs={}", &configs);
    Ok(configs)
}

#[command]
fn update_openai_config(_: State<'_, Mutex<AppState>>, configs: String) -> Result<String, ()> {
    println!("update openai config");
    let configs: Vec<OpenAiConfig> = serde_json::from_str(&configs).unwrap();
    let data = OpenAiConfigService::new().update(configs).unwrap();
    let configs = serde_json::to_string(&data).unwrap();
    println!("configs={}", &configs);
    Ok(configs)
}

#[command]
async fn sync_check(state: State<'_, Mutex<AppState>>) -> Result<bool, ()> {
    println!("sync check");
    let mut state = state.lock().await;
    if !state.login {
        return Ok(false);
    }

    // let m = state.member.clone();

    // 添加错误处理和重试机制
    match Executor::new()
        .auto_reply(state.client.client(), &Member::new(), &mut state.base)
        .await
    {
        Ok(login) => Ok(login),
        Err(e) => {
            println!("error={:?}", e);
            // 判断错误类型
            if let Some(err) = e.downcast_ref::<reqwest::Error>() {
                if err.is_connect() {
                    println!("网络连接失败（可能断网）");
                    state.login = false;
                    return Ok(false); // 断网继续重试
                }
            }
            Ok(true)
        }
    }
}

// #[command]
// async fn listen(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
//     let state = state.lock().await;
//     let arc_base = Arc::new(Mutex::new(state.base.clone()));
//     let next_arc_base = Arc::clone(&arc_base);
//
//     let (tx, mut rx) = mpsc::channel(1);
//     let mut handle = tokio::spawn(async move {
//         let mut executor = Executor::new();
//         loop {
//             let temp = Arc::clone(&arc_base);
//             let mut base = temp.lock().await;
//             println!("waiting for response");
//             let _ = executor
//                 .auto_reply(state.client.client(), &mut base)
//                 .await
//                 .unwrap();
//
//             tx.send(true).await.unwrap();
//             tokio::time::sleep(Duration::from_secs(1)).await;
//         }
//     });
//     loop {
//         // 设置超时时间
//         match timeout(Duration::from_secs(60), rx.recv()).await {
//             Ok(Some(_)) => {
//                 println!("Task has resumed running");
//             }
//             Ok(None) | Err(_) => {
//                 println!("Task did not resume in time. Trying to restart...");
//                 handle.abort();
//
//                 let (next_tx, next_rx) = mpsc::channel(1);
//                 rx = next_rx;
//                 let next_base = Arc::clone(&next_arc_base);
//                 let next_handle = tokio::spawn(async move {
//                     let mut executor = Executor::new();
//                     loop {
//                         println!("waiting for response");
//                         let temp = Arc::clone(&next_base);
//                         let mut base = temp.lock().await;
//
//                         let _ = executor
//                             .auto_reply(state.client.client(), &mut base)
//                             .await
//                             .unwrap();
//
//                         next_tx.send(true).await.unwrap();
//                         tokio::time::sleep(Duration::from_secs(1)).await;
//                     }
//                 });
//                 handle = next_handle;
//             }
//         }
//     }
// }

#[command]
fn create_scheduled_rule(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("create scheduled rule");
    let rule: ScheduledRule = serde_json::from_str(&config).unwrap();
    let rules = ScheduledRuleService::new().create(rule).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    Ok(configs)
}

#[command]
fn get_scheduled_rules(_: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("get scheduled rules");
    let rules = ScheduledRuleService::new().m_get().unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}

#[command]
fn del_scheduled_rule(_: State<'_, Mutex<AppState>>, id: String) -> Result<String, ()> {
    println!("del scheduled rule");
    let rules = ScheduledRuleService::new().del(id).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}

#[command]
fn update_scheduled_rule(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("update scheduled rule");
    let rule: ScheduledRule = serde_json::from_str(&config).unwrap();
    let rules = ScheduledRuleService::new().update(rule).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}
#[command]
async fn logout(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    println!("logout");
    let mut state = state.lock().await;
    state.login = false;
    state.base = Base::new();
    state.member = Member::new();
    state.client = WxOpenapiClient::new();
    state.uuid = String::default();
    Ok(())
}

#[command]
async fn redeem(state: State<'_, Mutex<AppState>>,token: String) -> Result<String, ()> {
    let mut state = state.lock().await;
    let resp = state.auth.redeem(token).await.unwrap();
    println!("redeem resp={:?}", &resp);
    Ok(serde_json::to_string(&resp).unwrap())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::new()));
            let root = app.path().resolve("resources", BaseDirectory::Resource)?;
            println!("root = {:?}", &root);
            APPDATA_PATH.lock().unwrap().clone_from(&root);
            create_dir_all(Path::new(&root).join("data")).unwrap();
            create_dir_all(Path::new(&root).join("image/avatar")).unwrap();
            create_dir_all(Path::new(&root).join("image/code")).unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            refresh,
            login_check,
            logout,
            get_members,
            get_auth,
            get_user,
            create_group,
            get_groups,
            del_group,
            create_auto_reply_rule,
            get_auto_reply_rules,
            del_auto_reply_rule,
            update_auto_reply_rule,
            create_openai,
            get_openai_list,
            del_openai,
            update_openai,
            test_openai,
            get_openai_config,
            update_openai_config,
            sync_check,
            create_scheduled_rule,
            get_scheduled_rules,
            del_scheduled_rule,
            update_scheduled_rule,
            switch_auto_reply_rule_status,
            redeem,
        ])
        .run(generate_context! {})
        .expect("error while running tauri application");
}
