// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod openapi;

use crate::openapi::base::Base;
use crate::openapi::login::Login;
use crate::openapi::member::Member;
use crate::openapi::message::MessageService;
use crate::openapi::rule::{
    Group, GroupService, OpenAi, OpenAiConfig, OpenAiConfigService, OpenAiService, Rule,
    RuleService, APPDATA_PATH,
};
use once_cell::sync::Lazy;
use openapi::client::WxOpenapiClient;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, time};
use tauri::path::BaseDirectory;
use tauri::{command, generate_context, Manager, State};
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;

struct AppState {
    uuid: String,
    login: bool,
    base: Base,
    member: Member,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            uuid: String::default(),
            login: false,
            base: Base::new(),
            member: Member::new(),
        }
    }
}

static WX_CLIENT: Lazy<WxOpenapiClient> = Lazy::new(|| WxOpenapiClient::new());

#[command]
async fn refresh(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    println!("[refresh][main] qr code");
    let mut state = state.lock().await;
    let mut login = Login::new();
    let uuid = login.get_uuid(WX_CLIENT.client()).await.unwrap();
    state.uuid = uuid.clone();
    let data = login.get_qr_code(WX_CLIENT.client(), &uuid).await.unwrap();
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
async fn check(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("[check][main] check login status");
    let mut state = state.lock().await;
    if state.login {
        return Ok("200".to_string());
    }
    let mut login = Login::new();
    let resp = login.check(WX_CLIENT.client(), &state.uuid).await.unwrap();
    println!("[check][main] resp={:?}", &resp);
    if resp.code == "200" {
        state.login = true;
        let base = Base::new()
            .init(WX_CLIENT.client(), &resp.redirect_uri)
            .await
            .unwrap();

        state.base = base.clone();
    }
    Ok(resp.code)
}

#[command]
async fn get_members(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("load wechat members");
    let mut state = state.lock().await;

    state.member = Member::new()
        .init(WX_CLIENT.client(), &state.base)
        .await
        .unwrap();

    let data = serde_json::to_string(&state.member.member_list).unwrap();
    println!("members={:?}", &data);
    Ok(data)
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
fn create_rule(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("create rule");
    let rule: Rule = serde_json::from_str(&config).unwrap();
    let rules = RuleService::new().create(rule).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    Ok(configs)
}

#[command]
fn get_rules(_: State<'_, Mutex<AppState>>) -> Result<String, ()> {
    println!("get rules");
    let rules = RuleService::new().m_get().unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}

#[command]
fn del_rule(_: State<'_, Mutex<AppState>>, id: String) -> Result<String, ()> {
    println!("del rule");
    let rules = RuleService::new().del(id).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
}

#[command]
fn update_rule(_: State<'_, Mutex<AppState>>, config: String) -> Result<String, ()> {
    println!("update rule");
    let rule: Rule = serde_json::from_str(&config).unwrap();
    let rules = RuleService::new().update(rule).unwrap();
    let configs = serde_json::to_string(&rules).unwrap();
    println!("rules={:?}", &rules);
    Ok(configs)
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
async fn listen(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut state = state.lock().await;
    let arc_base = Arc::new(Mutex::new(state.base.clone()));
    let next_arc_base = Arc::clone(&arc_base);

    let (tx, mut rx) = mpsc::channel(1);
    let mut handle = tokio::spawn(async move {
        let mut service = MessageService::new();

        loop {
            let temp = Arc::clone(&arc_base);
            let mut base = temp.lock().await;
            println!("waiting for response");
            service.sync(WX_CLIENT.client(), &mut base).await;
            tx.send(true).await.unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    loop {
        // 设置超时时间
        match timeout(Duration::from_secs(60), rx.recv()).await {
            Ok(Some(_)) => {
                println!("Task has resumed running");
            }
            Ok(None) | Err(_) => {
                println!("Task did not resume in time. Trying to restart...");
                handle.abort();

                let (next_tx, next_rx) = mpsc::channel(1);
                rx = next_rx;
                let next_base = Arc::clone(&next_arc_base);
                let next_handle = tokio::spawn(async move {
                    let mut service = MessageService::new();

                    loop {
                        println!("waiting for response");
                        let temp = Arc::clone(&next_base);
                        let mut base = temp.lock().await;
                        service.sync(WX_CLIENT.client(), &mut base).await;
                        next_tx.send(true).await.unwrap();
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                });
                handle = next_handle;
            }
        }
    }
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
            check,
            get_members,
            get_user,
            create_group,
            get_groups,
            del_group,
            create_rule,
            get_rules,
            del_rule,
            update_rule,
            create_openai,
            get_openai_list,
            del_openai,
            update_openai,
            test_openai,
            get_openai_config,
            update_openai_config,
            listen
        ])
        .run(generate_context! {})
        .expect("error while running tauri application");
}
