use crate::openapi::tool::str_tool::capture;
use crate::openapi::tool::time_tool::get_r;
use regex::Regex;
use reqwest::Client;
use std::error::Error;

#[warn(dead_code)]
pub struct Login {
    // params
    appid: String,
    pub uuid: String,

    // re
    uuid_re: Regex,
    code_re: Regex,
    redirect_uri_re: Regex,

    // uri
    uuid_uri: String,
    qrcode_uri: String,
    check_uri: String,
    pub(crate) redirect_uri: String,
}

#[derive(Debug)]
pub struct CheckResp {
    pub code: String,
    pub redirect_uri: String,
}

impl CheckResp {
    pub fn default() -> CheckResp {
        CheckResp {
            code: "".to_string(),
            redirect_uri: "".to_string(),
        }
    }
}

impl Login {
    pub(crate) fn new() -> Login {
        Login {
            appid: String::from("wx782c26e4c19acffb"),
            uuid: String::new(),
            uuid_re: Regex::new(r#"uuid\s*=\s*"(.*)""#).unwrap(),
            code_re: Regex::new(r#"code\s*=\s*(\d*)"#).unwrap(),
            redirect_uri_re: Regex::new(r#"redirect_uri\s*=\s*"(.*)""#).unwrap(),
            uuid_uri: String::from("https://login.wx.qq.com/jslogin"),
            qrcode_uri: String::from("https://login.wx.qq.com/qrcode"),
            check_uri: String::from("https://login.weixin.qq.com/cgi-bin/mmwebwx-bin/login"),
            redirect_uri: String::new(),
        }
    }

    pub async fn get_uuid(&mut self, cli: Client) -> Result<String, Box<dyn Error>> {
        let text = cli
            .get(self.uuid_uri.as_str())
            .query(&[("appid", self.appid.as_str())])
            .send()
            .await?
            .text()
            .await?;

        let uuid = capture(&self.uuid_re, text.as_str()).unwrap_or_default();
        self.uuid = uuid.clone();
        println!("[refresh][get_uuid] uuid:{}", uuid);
        Ok(uuid)
    }

    pub async fn get_qr_code(
        &mut self,
        cli: Client,
        uuid: &str,
    ) -> Result<bytes::Bytes, Box<dyn Error>> {
        let data = cli
            .get(format!("{}/{}", self.qrcode_uri, uuid))
            .header("Content-Type", "image/jpeg")
            .send()
            .await?
            .bytes()
            .await?;
        println!("[refresh][get_qr_code] size:{}", data.len());
        Ok(data)
    }

    pub async fn check(&mut self, cli: Client, uuid: &str) -> Result<CheckResp, Box<dyn Error>> {
        let params = get_r();
        let text = cli
            .get(self.check_uri.as_str())
            .query(&[
                ("loginicon", "true"),
                ("uuid", uuid),
                ("tip", "1"),
                ("r", params.0.as_str()),
                ("_", params.1.as_str()),
            ])
            .send()
            .await?
            .text()
            .await?;

        let code = capture(&self.code_re, text.as_str()).unwrap_or_default();
        Ok(CheckResp {
            code,
            redirect_uri: capture(&self.redirect_uri_re, text.as_str()).unwrap_or_default(),
        })
    }
}
