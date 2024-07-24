use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use lazy_static::lazy_static;
use quick_xml::events::Event;
use regex::Regex;
use reqwest::Client;
use reqwest_cookie_store::CookieStoreMutex;
use rocket::http::ext::IntoCollection;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_value, Value};
use xml_doc::Document;
use crate::openapi::tool::str_tool::capture;
use crate::openapi::tool::time_tool::get_r;

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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BaseRequest {
    pub skey: String,
    pub sid: String,
    pub uin: String,
    pub device_id: String,
}

impl BaseRequest {
    fn new() -> BaseRequest {
        BaseRequest {
            skey: "".to_string(),
            sid: "".to_string(),
            uin: "".to_string(),
            device_id: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BaseResponse {
    pub ret: i64,
    pub err_msg: String,
}

pub struct Base {
    // user params & config
    pub user: User,
    pub sync_key: SyncKey,
    pub base_request: BaseRequest,

    // re
    base_uri_re: Regex,

    // uri
    pub base_uri: String,
    pub file_uri: String,
    pub sync_uri: String,
}

/*
    "User": {
        "Uin": 2610361229,
        "UserName": "@1134150eec7f49e08717d1fa3747cc5c44b4f359fc730b08c1c1079ef9c6bc37",
        "NickName": "杨晓迪",
        "HeadImgUrl": "/cgi-bin/mmwebwx-bin/webwxgeticon?seq=2076678974&username=@1134150eec7f49e08717d1fa3747cc5c44b4f359fc730b08c1c1079ef9c6bc37&skey=@crypt_42213ffc_05d5b49951e998a0a0121c768d89d7b5",
        "RemarkName": "",
        "PYInitial": "",
        "PYQuanPin": "",
        "RemarkPYInitial": "",
        "RemarkPYQuanPin": "",
        "HideInputBarFlag": 0,
        "StarFriend": 0,
        "Sex": 1,
        "Signature": "让你开心一点，再让你开心一点",
        "AppAccountFlag": 0,
        "VerifyFlag": 0,
        "ContactFlag": 0,
        "WebWxPluginSwitch": 0,
        "HeadImgFlag": 1,
        "SnsFlag": 257
    },
*/
#[derive(Debug, Deserialize)]
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
            user_name: "".to_string(),
            nick_name: "".to_string(),
            head_img_url: "".to_string(),
            remark_name: "".to_string(),
            p_y_initial: "".to_string(),
            p_y_quan_pin: "".to_string(),
            remark_p_y_initial: "".to_string(),
            remark_p_y_quan_pin: "".to_string(),
            hide_input_bar_flag: 0,
            star_friend: 0,
            sex: 0,
            signature: "".to_string(),
            app_account_flag: 0,
            verify_flag: 0,
            contact_flag: 0,
            web_wx_plugin_switch: 0,
            head_img_flag: 0,
            sns_flag: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
struct InitResponse {
    base_response: BaseResponse,
    user: User,
    sync_key: SyncKey,
}

impl Base {
    pub fn new() -> Base {
        Base {
            user: User::new(),
            sync_key: SyncKey::new(),
            base_request: BaseRequest::new(),
            base_uri_re: Regex::new(r#"https?://([\w|\\.]*)/cgi-bin/mmwebwx-bin"#).unwrap(),
            base_uri: String::new(),
            file_uri: String::new(),
            sync_uri: String::new(),
        }
    }


    pub async fn init(&mut self, cli: &Client, cookie_store: &Arc<CookieStoreMutex>, redirect_uri: &str) -> Result<(), Box<dyn Error>> {
        // get sync and file uri
        let base = capture(&self.base_uri_re, redirect_uri).unwrap_or_default();


        let (file, sync) = URI_MAP.get(&base).unwrap();
        self.file_uri = format!("https://{}/cgi-bin/mmwebwx-bin", file);
        self.sync_uri = format!("https://{}/cgi-bin/mmwebwx-bin", sync);
        self.base_uri = format!("https://{}/cgi-bin/mmwebwx-bin", base);

        // fun=new&version=v2&mod=desktop&lang=zh_CN
        let data = cli.get(redirect_uri)
            .query(&[("fun", "new"), ("version", "v2"), ("mod", "desktop"), ("lang", "zh-CN")])
            .send().await?.text().await?;
        // {
        //     let store = cookie_store.lock().unwrap();
        //     for c in store.iter_any() {
        //         println!("{:?}", c);
        //     }
        // }

        let mut reader = quick_xml::Reader::from_str(&data);

        let mut event_name = String::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Start(e) => event_name = String::from_utf8(e.name().local_name().as_ref().to_vec()).unwrap(),
                Event::Text(e) => {
                    match event_name.as_str() {
                        "skey" => self.base_request.skey = e.unescape().unwrap().to_string(),
                        "wxsid" => self.base_request.sid = e.unescape().unwrap().to_string(),
                        "wxuin" => self.base_request.uin = e.unescape().unwrap().to_string(),
                        "pass_ticket" => self.base_request.device_id = e.unescape().unwrap().to_string(),
                        _ => (),
                    }
                }
                Event::Eof => break,
                _ => (),
            }
        }

        // web_init

        let r = get_r();
        let data = cli.post(format!("{}/webwxinit", self.base_uri))
            .query(&[("r", r.0.as_str()), ("pass_ticket", self.base_request.device_id.as_str())])
            .body(json!({
                "BaseRequest": self.base_request,
            }).to_string())
            .send()
            .await?
            .text().await?;


        let resp: InitResponse = from_str(&data).unwrap();

        self.user = resp.user;
        self.sync_key = resp.sync_key;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::str::from_utf8;
    use quick_xml::events::Event;
    use quick_xml::reader;
    use rocket::serde::json::to_value;
    use super::*;

    #[test]
    fn xml_parse() {
        let data = r#"<error><ret>0</ret><message></message><skey>@crypt_42213ffc_363f8334c042e4ddf9313cbbbdb37604</skey><wxsid>mFtkPR4mOTFq1/XP</wxsid><wxuin>2610361229</wxuin><pass_ticket>wwrXQck85TpkRx%2BzS8k%2FhS%2B%2FPtZJPPh7PuW1zEMvzoyOanjLRJ817GMXfLthKXpANhbGQCKM%2Febzp0A5pIwItA%3D%3D</pass_ticket><isgrayscale>1</isgrayscale></error>"#;
        let mut reader = quick_xml::Reader::from_str(data);

        let mut event_name = String::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Start(e) => event_name = String::from_utf8(e.name().local_name().as_ref().to_vec()).unwrap(),
                Event::Text(e) => {
                    match event_name.as_str() {
                        "skey" | "wxsid" | "wxuin" | "pass_ticket" => println!("{}:{}", event_name, &e.unescape().unwrap()),
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => (),
            }
        }
    }

    #[test]
    fn sync_key_join() {
        let sync_key: SyncKey = from_str("
        {
            \"count\": 4,
            \"list\":[
               {
                \"key\":0,
                \"value\":1234
               },
               {
                \"key\":1,
                \"value\":4321
               }
            ]
        }").unwrap();

        println!("{}", sync_key.to_string())
    }

    #[test]
    fn init_response() {
        let mut base = Base::new();

        base.sync_key.list=vec![SyncKeyItem{ key: 0, val: 0 },SyncKeyItem{ key: 1, val: 1 }];
        let json_str = json!({
                "BaseRequest":to_value(&base.base_request).unwrap(),
                "SyncKey": &base.sync_key.to_string()
            }).to_string();
        println!("{}", json_str)
    }
}