use crate::openapi::base::{Base, BaseResponse, SyncKey};
use crate::openapi::rule::OpenAiMessage;
use crate::openapi::tool::str_tool::capture;
use crate::openapi::tool::time_tool::{get_msg_id, get_r};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_value};
use std::collections::HashMap;
use std::error::Error;
use std::option::Option;
use std::sync::{Arc, Mutex};

static HISTORY: Lazy<Arc<Mutex<History>>> = Lazy::new(|| Arc::new(Mutex::new(History::new(10))));

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SyncResponse {
    pub base_response: BaseResponse,
    pub sync_check_key: SyncKey,
    add_msg_count: i64,
    pub add_msg_list: Vec<AddMsg>,
}

impl SyncResponse {
    pub fn new_err() -> SyncResponse {
        SyncResponse {
            base_response: BaseResponse {
                ret: -1,
                err_msg: "".to_string(),
            },
            sync_check_key: SyncKey::new(),
            add_msg_count: 0,
            add_msg_list: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddMsg {
    pub msg_id: String,
    pub from_user_name: String,
    pub to_user_name: String,
    pub msg_type: i64,
    pub content: String,
    pub status: i64,
    pub img_status: i64,
    pub create_time: i64,
    pub voice_length: i64,
    pub play_length: i64,
    pub file_name: String,
    pub file_size: String,
    pub media_id: String,
    pub url: String,
    pub app_msg_type: i64,
    pub status_notify_code: i64,
    pub status_notify_user_name: String,
    pub recommend_info: RecommendInfo,
    pub forward_flag: i64,
    pub app_info: AppInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AppInfo {
    app_i_d: String,
    r#type: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RecommendInfo {
    user_name: String,
    nick_name: String,
    q_q_num: i64,
    province: String,
    city: String,
    content: String,
    signature: String,
    alias: String,
    scene: i64,
    verify_flag: i64,
    attr_status: i64,
    sex: i64,
    ticket: String,
    op_code: i64,
}

pub struct History {
    max_num: usize,
    records: HashMap<String, Vec<OpenAiMessage>>,
}

impl History {
    pub fn new(max_num: usize) -> History {
        History {
            max_num,
            records: HashMap::new(),
        }
    }
    pub fn get(&mut self, user_name: &str) -> &Vec<OpenAiMessage> {
        if !self.records.contains_key(user_name) {
            self.records.insert(user_name.to_string(), vec![]);
        }
        self.records.get(user_name).unwrap()
    }
    pub fn push(&mut self, user_name: &str, messages: Vec<OpenAiMessage>) {
        let record = self.records.get_mut(user_name).unwrap();
        if record.len() >= self.max_num {
            record.remove(0);
            record.remove(0);
        }
        for message in messages {
            record.push(message);
        }
    }
}

pub struct MessageService {
    retcode_re: Regex,
    selector_re: Regex,
    pub history: History,
}

impl MessageService {
    pub fn new() -> MessageService {
        MessageService {
            retcode_re: Regex::new(r#"retcode:"(\d+)""#).unwrap(),
            selector_re: Regex::new(r#"selector:"(\d+)"}"#).unwrap(),
            history: History::new(10),
        }
    }

    pub async fn send_text_msg(
        &self,
        cli: Client,
        base: &Base,
        from_user: &str,
        to_users: Vec<&str>,
        content: &str,
    ) -> Result<(), Box<dyn Error>> {
        for to_user in to_users {
            let msg_id = get_msg_id();
            println!("msg_id={}", msg_id);

            let body = json!({
                "BaseRequest":to_value(&base.base_request).unwrap(),
                "Msg": {
                    "Type":1,
                    "Content":content,
                    "FromUserName":from_user,
                    "ToUserName":to_user,
                    "LocalID":msg_id,
                    "ClientMsgId":msg_id
                },
                "Scene":0
            })
            .to_string();
            let text = cli
                .post(format!("{}/webwxsendmsg", base.base_uri))
                .header("content-type", "application/json")
                .query(&[
                    ("lang", "zh-CN"),
                    ("pass_ticket", base.pass_ticket.as_str()),
                ])
                .body(body)
                .send()
                .await?
                .text()
                .await?;
            println!("text={}", text);
        }
        Ok(())
    }

    pub async fn check(&self, cli: Client, base: &Base) -> Result<Option<String>, Box<dyn Error>> {
        let r = get_r();
        let text = cli
            .get(format!("{}/synccheck", base.sync_uri))
            .query(&[
                ("r", r.0.as_str()),
                ("skey", base.base_request.skey.as_str()),
                ("sid", base.base_request.sid.as_str()),
                ("uin", base.base_request.uin.to_string().as_str()),
                ("deviceid", base.base_request.device_id.as_str()),
                ("synckey", base.sync_key.to_string().as_str()),
            ])
            .send()
            .await?
            .text()
            .await?;
        println!("text={}", text);

        Ok(capture(&self.retcode_re, &text))
    }

    pub async fn sync(&self, cli: Client, base: &Base) -> Result<SyncResponse, Box<dyn Error>> {
        let r = get_r();

        let rr = !r.1.parse::<i64>().unwrap();

        let data = cli
            .post(format!("{}/webwxsync", base.base_uri))
            .query(&[
                ("sid", base.base_request.sid.as_str()),
                ("skey", base.base_request.skey.as_str()),
                ("pass_ticket", base.pass_ticket.as_str()),
            ])
            .body(
                json!({
                    "BaseRequest":to_value(&base.base_request).unwrap(),
                    "SyncKey": to_value(&base.sync_key).unwrap(),
                    "rr": rr
                })
                .to_string(),
            )
            .send()
            .await?
            .text()
            .await?;

        let sync_response: SyncResponse = from_str(&data).unwrap_or(SyncResponse::new_err());
        Ok(sync_response)
    }
}
