use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::time::{Duration, SystemTime};
use regex::Regex;
use reqwest::Client;
use rocket::futures::TryFutureExt;
use rocket::serde::{Deserialize, Serialize};
use serde::__private::de::Content;
use serde_json::{from_str, json, to_value};
use tokio::time;
use crate::openapi::base::{Base, BaseResponse, SyncKey};
use crate::openapi::rule::{RuleType, TextMessageRule};
use crate::openapi::tool::str_tool::capture;
use crate::openapi::tool::time_tool::{get_msg_id, get_r};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SyncResponse {
    base_response: BaseResponse,
    sync_check_key: SyncKey,
    add_msg_count: i64,
    add_msg_list: Vec<AddMsg>,
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
struct AddMsg {
    msg_id: String,
    from_user_name: String,
    to_user_name: String,
    msg_type: i64,
    content: String,
    status: i64,
    img_status: i64,
    create_time: i64,
    voice_length: i64,
    play_length: i64,
    file_name: String,
    file_size: String,
    media_id: String,
    url: String,
    app_msg_type: i64,
    status_notify_code: i64,
    status_notify_user_name: String,
    recommend_info: RecommendInfo,
    forward_flag: i64,
    app_info: AppInfo,
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

pub struct Message {
    text_message_rules: HashMap<String, TextMessageRule>,
    sync_key: String,
    selector_re: Regex,
    alive: bool,
}

impl Message {
    pub fn new() -> Message {
        Message {
            text_message_rules: HashMap::new(),
            sync_key: String::new(),
            selector_re: Regex::new(r#"selector:"(\d+)"}"#).unwrap(),
            alive: true,
        }
    }

    pub async fn send_text_msg(&self, cli: &Client, base: &Base, from_user: &str, to_user: &str, content: &str) -> Result<(), Box<dyn Error>> {

        // url = '%s/webwxsendmsg' % self.loginInfo['url']
        // data = {
        //     'BaseRequest': self.loginInfo['BaseRequest'],
        //     'Msg': {
        //         'type': msgType,
        //         'Content': content,
        //         'FromUserName': self.storageClass.userName,
        //         'ToUserName': (toUserName if toUserName else self.storageClass.userName),
        //         'LocalID': int(time.time() * 1e4),
        //         'ClientMsgId': int(time.time() * 1e4),
        //     },
        //     'Scene': 0, }
        // headers = { 'ContentType': 'application/json; charset=UTF-8', 'User-Agent' : config.USER_AGENT }
        // r = self.s.post(url, headers=headers,
        //                 data=json.dumps(data, ensure_ascii=False).encode('utf8'))
        // return ReturnValue(rawResponse=r)


        let msg_id = get_msg_id();
        println!("msg_id={}", msg_id);
        let text = cli.post(format!("{}/webwxsendmsg", base.base_uri))
            .query(&[
                ("pass_ticket", base.base_request.device_id.as_str())
            ])
            .body(json!({
                "BaseRequest":to_value(&base.base_request).unwrap(),
                "Msg": {
                    "type":1,
                    "Content":content,
                    "FromUserName":from_user,
                    "ToUserName":to_user,
                    "LocalID":msg_id,
                    "ClientMsgId":msg_id
                },
                "Scene":0
            }).to_string())
            .send().await?.text().await?;
        println!("{}", text);

        Ok(())
    }

    pub fn register_rule(&mut self, rule_type: RuleType, rule_name: &str, rule_data: &str) {
        match rule_type {
            RuleType::Text => {
                let rule: TextMessageRule = from_str(rule_data).unwrap();
                self.text_message_rules.insert(rule_name.to_string(), rule);
            }
        }
    }

    async fn sync_check(&self, cli: &Client, base: &Base) -> Result<Option<String>, Box<dyn Error>> {
        let r = get_r();
        let text = cli.get(format!("{}/synccheck", base.sync_uri))
            .query(&[("r", r.0.as_str()),
                ("skey", base.base_request.skey.as_str()),
                ("sid", base.base_request.sid.as_str()),
                ("uin", base.base_request.uin.as_str()),
                ("deviceid", base.base_request.device_id.as_str()),
                ("synckey", base.sync_key.to_string().as_str())
            ])
            .send()
            .await?
            .text()
            .await?;

        Ok(capture(&self.selector_re, &text))
    }

    async fn sync(&self, cli: &Client, base: &mut Base) -> Result<(), Box<dyn Error>> {
        let r = get_r();

        let rr = !r.1.parse::<i64>().unwrap();

        let data = cli.post(format!("{}/webwxsync", base.base_uri))
            .query(&[("sid", base.base_request.sid.as_str()), ("skey", base.base_request.skey.as_str()), ("pass_ticket", base.base_request.device_id.as_str())])
            .body(json!({
                "BaseRequest":to_value(&base.base_request).unwrap(),
                "SyncKey": to_value(&base.sync_key).unwrap(),
                "rr": rr
            }).to_string())
            .send()
            .await?
            .text()
            .await?;

        let sync_response: SyncResponse = from_str(&data).unwrap_or(SyncResponse::new_err());

        if sync_response.base_response.ret < 0 {
            println!("webwxsync data={}", data);
            return Ok(());
        }

        // println!("sync_response={}", serde_json::to_string(&sync_response).unwrap());

        base.sync_key = sync_response.sync_check_key;


        for msg in sync_response.add_msg_list.iter() {
            // println!("msg={}", serde_json::to_string(msg).unwrap());
            match msg.msg_type {
                1 => {
                    // to msg.from_user_name

                    if msg.to_user_name == base.user.user_name {
                        println!("msg.from_user_name={}, msg.to_user_name={}, content={}", msg.from_user_name, msg.to_user_name, msg.content);
                        self.send_text_msg(cli, base, &msg.to_user_name, &msg.from_user_name, "【自动回复】用户在忙，稍后回复您~").await?
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn listen(&mut self, cli: &Client, base: &mut Base) {
        loop {
            let result = self.sync_check(cli, base).await.unwrap();
            match result.unwrap_or_default().as_str() {
                "" => {
                    self.alive = false
                }
                // "0" => {
                //     todo!()
                // }
                _ => {
                    self.sync(cli, base).await.expect("TODO: panic message");
                }
            }
            time::sleep(Duration::from_secs(1)).await;
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::from_str;
    use crate::openapi::message::SyncResponse;

    #[test]
    fn from_str_to_sync_resp() {
        let text = "
        {\"BaseResponse\":{\"Ret\":0,\"ErrMsg\":\"\"},\"AddMsgCount\":1,\"AddMsgList\":[{\"MsgId\":\"487567706613371358\",\"FromUserName\":\"@afe9545e415bf0b8d976c7201a88403c34fd7b535b796e35340962e82771960a\",\"ToUserName\":\"@c12c96a9bd844af30f1098ee05cf4885c7d6f55a7bacb5eddad033d241a83fb7\",\"MsgType\":1,\"Content\":\"收拾\",\"Status\":3,\"ImgStatus\":1,\"CreateTime\":1714475900,\"VoiceLength\":0,\"PlayLength\":0,\"FileName\":\"\",\"FileSize\":\"\",\"MediaId\":\"\",\"Url\":\"\",\"AppMsgType\":0,\"StatusNotifyCode\":0,\"StatusNotifyUserName\":\"\",\"RecommendInfo\":{\"UserName\":\"\",\"NickName\":\"\",\"QQNum\":0,\"Province\":\"\",\"City\":\"\",\"Content\":\"\",\"Signature\":\"\",\"Alias\":\"\",\"Scene\":0,\"VerifyFlag\":0,\"AttrStatus\":0,\"Sex\":0,\"Ticket\":\"\",\"OpCode\":0},\"ForwardFlag\":0,\"AppInfo\":{\"AppID\":\"\",\"Type\":0},\"HasProductId\":0,\"Ticket\":\"\",\"ImgHeight\":0,\"ImgWidth\":0,\"SubMsgType\":0,\"NewMsgId\":487567706613371358,\"OriContent\":\"\",\"EncryFileName\":\"\"}],\"ModContactCount\":0,\"ModContactList\":[],\"DelContactCount\":0,\"DelContactList\":[],\"ModChatRoomMemberCount\":0,\"ModChatRoomMemberList\":[]}";

        let resp: SyncResponse = from_str(text).unwrap();

        println!("{}", resp.add_msg_count);
    }
}