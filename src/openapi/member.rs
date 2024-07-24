use std::error::Error;
use std::path::Display;
use serde::Deserialize;
use serde_json::from_str;
use crate::openapi::base::{Base, BaseResponse};
use crate::openapi::tool::time_tool::get_r;

pub struct Member {
    member_list: Vec<ContactMember>,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContactMember {
    uin: i64,
    user_name: String,
    nick_name: String,
    head_img_url: String,
    contact_flag: i64,
    member_count: i64,
    member_list: Vec<ContactMember>,
    remark_name: String,
    hide_input_bar_flag: i64,
    sex: i64,
    signature: String,
    verify_flag: i64,
    owner_uin: i64,
    p_y_quan_pin: String,
    remark_p_y_initial: String,
    remark_p_y_quan_pin: String,
    star_friend: i64,
    app_account_flag: i64,
    statues: i64,
    province: String,
    city: String,
    alias: String,
    sns_flag: i64,
    uni_friend: i64,
    display_name: String,
    chat_room_id: i64,
    key_word: String,
    encry_chat_room_id: String,
    is_owner: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContactResponse {
    base_response: BaseResponse,
    member_count: i64,
    member_list: Vec<ContactMember>,
}

impl Member {
    pub fn new() -> Member {
        Member {
            member_list: Vec::new(),
        }
    }

    pub async fn init(&mut self, cli: &reqwest::Client, base: &Base) -> Result<(), Box<dyn Error>> {
        let params = get_r();
        //webwxgetcontact
        let data = cli.get(format!("{}/webwxgetcontact", base.base_uri))
            .query(&[("r", params.0.as_str()), ("seq", "0"), ("pass_ticket", base.base_request.device_id.as_str()), ("skey", base.base_request.skey.as_str()), ("target", "t")])
            .send()
            .await?
            .text()
            .await?;

        let resp: ContactResponse = from_str(&data).unwrap();
        self.member_list = resp.member_list;
        Ok(())
    }
}