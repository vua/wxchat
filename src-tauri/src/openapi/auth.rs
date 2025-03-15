use crate::openapi::rule::{StatusInfo, APPDATA_PATH};
use crate::openapi::tool::file_tool;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tokio::io;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    expired_time: i64,
    expired_html: String,
    token_expired_time: HashMap<String, i64>,
}

impl AuthConfig {
    pub fn new() -> AuthConfig {
        AuthConfig {
            expired_time: 0,
            expired_html: "".to_string(),
            token_expired_time: HashMap::new(),
        }
    }
}

pub struct Auth {
    client: Client,
    data_path: PathBuf,
    expired_time: i64,
    expired_html: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RedeemResp {
    expired_time: i64,
    status_info: StatusInfo,
}

impl Auth {
    pub fn new() -> Self {
        Auth {
            client: Client::new(),
            data_path: APPDATA_PATH
                .lock()
                .unwrap()
                .join("data/user/auth_token.json"),
            expired_time: 0,
            expired_html: "".to_string(),
        }
    }

    pub async fn init(mut self) -> Result<Auth, Box<dyn Error>> {
        let mut config = self.get_config().await.unwrap_or(AuthConfig::new());
        let token = self.get_token().unwrap_or(None);
        let expired_time = config.expired_time;
        let mut token_expired_time = 0;
        match token {
            Some(token) => {
                token_expired_time = config
                    .token_expired_time
                    .get(token.as_str())
                    .unwrap_or(&0)
                    .clone();
            }
            None => {}
        }
        self.expired_time = max(expired_time, token_expired_time);
        self.expired_html = config.expired_html.clone();
        Ok(self)
    }

    fn get_token(&self) -> io::Result<Option<String>> {
        let token = file_tool::get_or_create_file(self.data_path.as_path())?;
        if token.is_empty() {
            return Ok(None);
        }
        Ok(Some(token))
    }

    async fn get_config(&self) -> Result<AuthConfig, reqwest::Error> {
        let config = self
            .client
            .get("https://raw.githubusercontent.com/vua/wxchat/main/config.json")
            .send()
            .await?
            .json::<AuthConfig>()
            .await?;
        Ok(config)
    }

    pub fn authorize(&self) -> bool {
        self.expired_time > Utc::now().timestamp()
    }

    pub fn expired_time(&self) -> i64 {
        self.expired_time
    }

    pub fn message(&self) -> String {
        self.expired_html.clone()
    }

    pub async fn redeem(&mut self, token: String) -> Result<RedeemResp, Box<dyn Error>> {
        let auth_config = self.get_config().await.unwrap_or(AuthConfig::new());

        self.expired_html = auth_config.expired_html.clone();
        match auth_config.token_expired_time.get(&token) {
            Some(token_expired_time) => {
                if token_expired_time.clone() < Utc::now().timestamp() {
                    return Ok(RedeemResp {
                        expired_time: self.expired_time,
                        status_info: StatusInfo::new(-1, "兑换码已过期".to_string()),
                    });
                }
                self.expired_time = max(self.expired_time, token_expired_time.clone());
                fs::write(self.data_path.as_path(), token)?;

                Ok(RedeemResp {
                    expired_time: self.expired_time,
                    status_info: StatusInfo::new(0, "兑换成功".to_string()),
                })
            }
            None => Ok(RedeemResp {
                expired_time: self.expired_time,
                status_info: StatusInfo::new(-1, "无效的兑换码".to_string()),
            }),
        }
    }
}
