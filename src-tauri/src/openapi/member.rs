use crate::openapi::base::{get_head_img, Base, BaseResponse, User};
use crate::openapi::rule::GroupService;
use crate::openapi::tool::time_tool::get_r;
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone, Debug)]
pub struct Member {
    pub member_list: Vec<ContactMember>,
    pub member_map: HashMap<String, ContactMember>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContactMember {
    pub uin: i64,
    pub user_name: String,
    pub nick_name: String,
    pub head_img_url: String,
    pub contact_flag: i64,
    pub member_count: i64,
    pub member_list: Vec<ContactMember>,
    pub remark_name: String,
    pub hide_input_bar_flag: i64,
    pub sex: i64,
    pub signature: String,
    pub verify_flag: i64,
    pub owner_uin: i64,
    pub p_y_quan_pin: String,
    pub remark_p_y_initial: String,
    pub remark_p_y_quan_pin: String,
    pub star_friend: i64,
    pub app_account_flag: i64,
    pub statues: i64,
    pub province: String,
    pub city: String,
    pub alias: String,
    pub sns_flag: i64,
    pub uni_friend: i64,
    pub display_name: String,
    pub chat_room_id: i64,
    pub key_word: String,
    pub encry_chat_room_id: String,
    pub is_owner: i64,
}

impl ContactMember {
    pub fn from(user: &User) -> ContactMember {
        ContactMember {
            uin: user.uin,
            user_name: user.user_name.clone(),
            nick_name: user.nick_name.clone(),
            head_img_url: user.head_img_url.clone(),
            contact_flag: user.contact_flag,
            member_count: 0,
            member_list: Vec::new(),
            remark_name: user.remark_name.clone(),
            hide_input_bar_flag: user.hide_input_bar_flag,
            sex: user.sex,
            signature: user.signature.clone(),
            verify_flag: user.verify_flag,
            owner_uin: 0,
            p_y_quan_pin: user.user_name.clone(),
            remark_p_y_initial: user.remark_p_y_initial.clone(),
            remark_p_y_quan_pin: user.remark_p_y_quan_pin.clone(),
            star_friend: user.star_friend,
            app_account_flag: user.app_account_flag,
            statues: 0,
            province: "".to_string(),
            city: "".to_string(),
            alias: "".to_string(),
            sns_flag: user.sns_flag,
            uni_friend: 0,
            display_name: "".to_string(),
            chat_room_id: 0,
            key_word: "".to_string(),
            encry_chat_room_id: "".to_string(),
            is_owner: 0,
        }
    }
}

impl Clone for ContactMember {
    fn clone(&self) -> Self {
        // 实现克隆逻辑...
        // 注意：这里的实现必须确保所有字段都能被安全地克隆。
        ContactMember {
            uin: self.uin,
            user_name: self.user_name.clone(),
            nick_name: self.nick_name.clone(),
            head_img_url: self.head_img_url.clone(),
            contact_flag: self.contact_flag,
            member_count: self.member_count,
            member_list: self.member_list.clone(),
            remark_name: self.remark_name.clone(),
            hide_input_bar_flag: self.hide_input_bar_flag,
            sex: self.sex,
            signature: self.signature.clone(),
            verify_flag: self.verify_flag,
            owner_uin: self.owner_uin,
            p_y_quan_pin: self.p_y_quan_pin.clone(),
            remark_p_y_initial: self.remark_p_y_initial.clone(),
            remark_p_y_quan_pin: self.remark_p_y_quan_pin.clone(),
            star_friend: self.star_friend,
            app_account_flag: self.app_account_flag,
            statues: self.statues,
            province: self.province.clone(),
            city: self.city.clone(),
            alias: self.alias.clone(),
            sns_flag: self.sns_flag,
            uni_friend: self.uni_friend,
            display_name: self.display_name.clone(),
            chat_room_id: self.chat_room_id,
            key_word: self.key_word.clone(),
            encry_chat_room_id: self.encry_chat_room_id.clone(),
            is_owner: self.is_owner,
        }
    }
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
            member_map: HashMap::new(),
        }
    }

    pub fn get(&self, user_name: &str) -> Option<&ContactMember> {
        self.member_map.get(user_name)
    }

    pub async fn init(mut self, cli: Client, base: &Base) -> Result<Member, Box<dyn Error>> {
        // let dir_path = PathBuf::from("statics/image/avatar");
        // tool::file_tool::remove_files_in_dir(&dir_path).unwrap();
        let params = get_r();
        let data = cli
            .get(format!("{}/webwxgetcontact", base.base_uri))
            .query(&[
                ("r", params.0.as_str()),
                ("seq", "0"),
                ("pass_ticket", base.base_request.device_id.as_str()),
                ("skey", base.base_request.skey.as_str()),
                ("target", "t"),
            ])
            .send()
            .await?
            .text()
            .await?;

        let resp: ContactResponse = from_str(&data).unwrap();
        let mut handles = vec![];
        let semaphore = Arc::new(Semaphore::new(10));

        for member in resp.member_list {
            let client = cli.clone();
            let username = member.user_name.clone();
            let pyquanpin = member.p_y_quan_pin.clone();
            let _base = base.clone();
            self.member_map
                .insert(member.user_name.to_string(), member.clone());
            self.member_list.push(member);
            let permit = semaphore.clone().acquire_owned().await?;
            // get_head_img(client, &_base, username.as_str(), pyquanpin.as_str()).await.unwrap_or_default();
            handles.push(tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) =
                    get_head_img(client, &_base, username.as_str(), pyquanpin.as_str()).await
                {
                    eprintln!("Error fetching head image: {}", e);
                }
            }));
        }

        self.member_list.push(ContactMember::from(&base.user));
        // for handle in handles {
        //     handle.await?;
        // }
        join_all(handles).await;

        GroupService::new().update_by_member(&self.member_list)?;
        Ok(self)
    }
}
