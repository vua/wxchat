use crate::openapi::rule::APPDATA_PATH;
use crate::openapi::tool::str_tool::capture;
use crate::openapi::tool::time_tool::get_r;
use lazy_static::lazy_static;
use quick_xml::events::Event;
use rand::distributions::Uniform;
use rand::Rng;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;

lazy_static! {
    static ref URI_MAP: HashMap<String, (String, String)> = {
        let mut uri_map: HashMap<String, (String, String)> = HashMap::new();
        // init uri_map
        uri_map.insert(String::from("wx2.qq.com"), (String::from("file.wx2.qq.com"), String::from("webpush.wx2.qq.com")));
        uri_map.insert(String::from("wx8.qq.com"), (String::from("file.wx8.qq.com"), String::from("webpush.wx8.qq.com")));
        uri_map.insert(String::from("wx.qq.com"), (String::from("file.wx.qq.com"), String::from("webpush.wx.qq.com")));
        uri_map.insert(String::from("qq.com"), (String::from("file.wx.qq.com"), String::from("webpush.wx.qq.com")));
        uri_map.insert(String::from("web2.wechat.com"), (String::from("file.web2.wechat.com"), String::from("webpush.web2.wechat.com")));
        uri_map.insert(String::from("web.wechat.com"), (String::from("file.web.wechat.com"), String::from("webpush.web.wechat.com")));
        uri_map.insert(String::from("wechat.com"), (String::from("file.web.wechat.com"), String::from("webpush.web.wechat.com")));
        uri_map
    };
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BaseRequest {
    pub skey: String,
    pub sid: String,
    pub uin: i64,
    #[serde(rename = "DeviceID")]
    pub device_id: String, // pass_ticket
}

impl BaseRequest {
    pub fn new() -> BaseRequest {
        let digits = rand::thread_rng()
            .sample_iter(&Uniform::from('0'..'9'))
            .take(15)
            .collect::<Vec<char>>()
            .iter()
            .collect::<String>();

        BaseRequest {
            skey: String::new(),
            sid: String::new(),
            uin: 0,
            device_id: format!("e{}", digits),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BaseResponse {
    pub ret: i64,
    pub err_msg: String,
}

impl BaseResponse {
    fn new() -> BaseResponse {
        BaseResponse {
            ret: 0,
            err_msg: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Base {
    pub base_response: BaseResponse,
    pub user: User,
    pub sync_key: SyncKey,
    pub base_request: BaseRequest,
    pub base_uri: String,
    pub sync_uri: String,
    pub file_uri: String,
    pub pass_ticket: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    uin: i64,
    pub user_name: String,
    nick_name: String,
    head_img_url: String,
    remark_name: String,
    p_y_initial: String,
    p_y_quan_pin: String,
    remark_p_y_initial: String,
    remark_p_y_quan_pin: String,
    hide_input_bar_flag: i64,
    star_friend: i64,
    sex: i64,
    signature: String,
    app_account_flag: i64,
    verify_flag: i64,
    contact_flag: i64,
    web_wx_plugin_switch: i64,
    head_img_flag: i64,
    sns_flag: i64,
}

impl User {
    fn new() -> User {
        User {
            uin: 0,
            user_name: String::new(),
            nick_name: String::new(),
            head_img_url: String::new(),
            remark_name: String::new(),
            p_y_initial: String::new(),
            p_y_quan_pin: String::new(),
            remark_p_y_initial: String::new(),
            remark_p_y_quan_pin: String::new(),
            hide_input_bar_flag: 0,
            star_friend: 0,
            sex: 0,
            signature: String::new(),
            app_account_flag: 0,
            verify_flag: 0,
            contact_flag: 0,
            web_wx_plugin_switch: 0,
            head_img_flag: 0,
            sns_flag: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SyncKey {
    pub count: i64,
    pub list: Vec<SyncKeyItem>,
}

impl SyncKey {
    pub fn new() -> SyncKey {
        SyncKey {
            count: 0,
            list: vec![],
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();

        for (i, item) in self.list.iter().enumerate() {
            result.push_str(&item.to_string());
            if i != self.list.len() {
                result.push_str("|")
            }
        }

        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SyncKeyItem {
    key: i64,
    val: i64,
}

impl SyncKeyItem {
    pub fn to_string(&self) -> String {
        format!("{}_{}", self.key, self.val)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InitResp {
    base_response: BaseResponse,
    user: User,
    sync_key: SyncKey,
}

impl InitResp {
    pub fn new() -> InitResp {
        InitResp {
            base_response: BaseResponse::new(),
            user: User::new(),
            sync_key: SyncKey::new(),
        }
    }
}

impl Base {
    pub fn new() -> Base {
        Base {
            base_response: BaseResponse::new(),
            user: User::new(),
            sync_key: SyncKey::new(),
            base_request: BaseRequest::new(),
            base_uri: String::new(),
            sync_uri: String::new(),
            file_uri: String::new(),
            pass_ticket: String::new(),
        }
    }

    pub async fn init(mut self, cli: Client, redirect_uri: &str) -> Result<Base, Box<dyn Error>> {
        // get sync and file uri
        let base = capture(
            &Regex::new(r#"https?://([\w|\\.]*)/cgi-bin/mmwebwx-bin"#).unwrap(),
            redirect_uri,
        )
        .unwrap_or_default();

        let (file, sync) = URI_MAP.get(&base).unwrap();
        self.file_uri = format!("https://{}/cgi-bin/mmwebwx-bin", file);
        self.sync_uri = format!("https://{}/cgi-bin/mmwebwx-bin", sync);
        self.base_uri = format!("https://{}/cgi-bin/mmwebwx-bin", base);

        let data = cli
            .get(redirect_uri)
            .query(&[
                ("fun", "new"),
                ("version", "v2"),
                ("mod", "desktop"),
                ("lang", "zh-CN"),
            ])
            .send()
            .await?
            .text()
            .await?;

        println!("data={}", data);

        let mut reader = quick_xml::Reader::from_str(&data);

        let mut event_name = String::new();

        loop {
            match reader.read_event().unwrap() {
                Event::Start(e) => {
                    event_name = String::from_utf8(e.name().local_name().as_ref().to_vec()).unwrap()
                }
                Event::Text(e) => match event_name.as_str() {
                    "skey" => {
                        self.base_request.skey = e.unescape().unwrap().to_string();
                    }
                    "wxsid" => self.base_request.sid = e.unescape().unwrap().to_string(),
                    "wxuin" => {
                        self.base_request.uin =
                            e.unescape().unwrap().to_string().parse::<i64>().unwrap()
                    }
                    "pass_ticket" => self.pass_ticket = e.unescape().unwrap().to_string(),

                    _ => (),
                },
                Event::Eof => break,
                _ => (),
            }
        }

        // web_init
        let r = get_r();
        let data = cli
            .post(format!("{}/webwxinit", self.base_uri))
            .query(&[
                ("r", r.0.as_str()),
                ("pass_ticket", self.pass_ticket.as_str()),
            ])
            .body(
                json!({
                    "BaseRequest": self.base_request,
                })
                .to_string(),
            )
            .send()
            .await?
            .text()
            .await?;

        let resp: InitResp = from_str(&data).unwrap();

        self.base_response = resp.base_response;
        self.user = resp.user;
        self.sync_key = resp.sync_key;
        get_head_img(
            cli.clone(),
            &self.clone(),
            &self.user.user_name,
            &self.user.user_name,
        )
        .await?;
        Ok(self)
    }
}

pub async fn get_head_img(
    cli: Client,
    base: &Base,
    username: &str,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let path = APPDATA_PATH
        .lock()
        .unwrap()
        .join(format!("image/avatar/{}.jpg", filename));

    if path.exists() {
        return Ok(());
    }
    println!("[get_head_img] {}", &path.to_str().unwrap());
    let data = cli
        .get(format!("{}/webwxgeticon", base.base_uri))
        .query(&[
            ("seq", "0"),
            ("username", username),
            ("skey", base.base_request.skey.as_str()),
            ("type", "big"),
            ("target", "t"),
        ])
        .send()
        .await?
        .bytes()
        .await?;
    let mut file = File::create(&path).unwrap();
    file.write_all(&data[..]).unwrap();
    file.flush().unwrap();

    Ok(())
}
