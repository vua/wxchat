use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use regex::Regex;
use reqwest::Client;
use crate::openapi::tool::str_tool::capture;
use crate::openapi::tool::time_tool::get_r;

pub struct Login {
    // params
    appid: String,
    uuid: String,

    //re
    uuid_re: Regex,
    code_re: Regex,
    redirect_uri_re: Regex,

    // uri
    uuid_uri: String,
    qr_uri: String,
    check_uri: String,
    pub(crate) redirect_uri: String,
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
            qr_uri: String::from("https://login.wx.qq.com/qrcode"),
            check_uri: String::from("https://login.weixin.qq.com/cgi-bin/mmwebwx-bin/login"),
            redirect_uri: String::new(),
        }
    }

    pub async fn get_qr_code_uuid(&mut self, cli: &Client) -> Result<String, Box<dyn Error>> {
        let text = cli.get(self.uuid_uri.as_str())
            .query(&[("appid", self.appid.as_str())])
            .send()
            .await?
            .text()
            .await?;

        let uuid = capture(&self.uuid_re, text.as_str()).unwrap_or_default();
        self.uuid = uuid.clone();
        Ok(uuid)
    }

    pub async fn get_qr_code(&self, cli: &Client) -> Result<(), Box<dyn Error>> {
        let data = cli.get(format!("{}/{}", self.qr_uri, self.uuid))
            .header("Content-Type", "image/jpeg")
            .send()
            .await?
            .bytes()
            .await?;
        let mut file = File::create("./qr_code.jpg").unwrap();
        file.write_all(&data[..]).expect("write_all fail");
        file.flush().expect("flush fail");
        Ok(())
    }

    pub async fn check(&mut self, cli: &Client) -> Result<bool, Box<dyn Error>> {
        let local_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let params = get_r();
        let text = cli.get(self.check_uri.as_str())
            .query(&[("loginicon", "true"), ("uuid", self.uuid.as_str()), ("tip", "1"), ("r", params.0.as_str()), ("_", params.1.as_str())])
            .send()
            .await?
            .text()
            .await?;

        let code = capture(&self.code_re, text.as_str()).unwrap_or_default();
        if code == "200" {
            let redirect_uri = capture(&self.redirect_uri_re, text.as_str()).unwrap_or_default();
            self.redirect_uri = redirect_uri;
        }
        Ok(code == "200")
    }
}