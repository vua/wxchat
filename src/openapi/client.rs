use std::error::Error;
use std::sync::Arc;

use reqwest;
use regex::{Regex};

use std::time::{Duration};
use reqwest::header::HeaderMap;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use tokio::time;
use crate::openapi::base::Base;
use crate::openapi::login::Login;
use crate::openapi::member::Member;
use crate::openapi::message::Message;


#[warn(dead_code)]
pub struct WxOpenapiClient {
    cli: reqwest::Client,
    cookie_store: Arc<CookieStoreMutex>,
    // re
    ticket_re: Regex,
    // mod
    login: Login,
    base: Base,
    member: Member,
    pub message: Message,
}

impl WxOpenapiClient {
    pub fn new() -> WxOpenapiClient {
        let cookie_store = CookieStore::default();
        let cookie_store = CookieStoreMutex::new(cookie_store);
        let cookie_store = Arc::new(cookie_store);

        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert("extspam",
                       "Go8FCIkFEokFCggwMDAwMDAwMRAGGvAESySibk50w5Wb3uTl2c2h64jVVrV7gNs06GFlWplHQbY/5FfiO++1yH4ykCyNPWKXmco+wfQzK5R98D3so7rJ5LmGFvBLjGceleySrc3SOf2Pc1gVehzJgODeS0lDL3/I/0S2SSE98YgKleq6Uqx6ndTy9yaL9qFxJL7eiA/R3SEfTaW1SBoSITIu+EEkXff+Pv8NHOk7N57rcGk1w0ZzRrQDkXTOXFN2iHYIzAAZPIOY45Lsh+A4slpgnDiaOvRtlQYCt97nmPLuTipOJ8Qc5pM7ZsOsAPPrCQL7nK0I7aPrFDF0q4ziUUKettzW8MrAaiVfmbD1/VkmLNVqqZVvBCtRblXb5FHmtS8FxnqCzYP4WFvz3T0TcrOqwLX1M/DQvcHaGGw0B0y4bZMs7lVScGBFxMj3vbFi2SRKbKhaitxHfYHAOAa0X7/MSS0RNAjdwoyGHeOepXOKY+h3iHeqCvgOH6LOifdHf/1aaZNwSkGotYnYScW8Yx63LnSwba7+hESrtPa/huRmB9KWvMCKbDThL/nne14hnL277EDCSocPu3rOSYjuB9gKSOdVmWsj9Dxb/iZIe+S6AiG29Esm+/eUacSba0k8wn5HhHg9d4tIcixrxveflc8vi2/wNQGVFNsGO6tB5WF0xf/plngOvQ1/ivGV/C1Qpdhzznh0ExAVJ6dwzNg7qIEBaw+BzTJTUuRcPk92Sn6QDn2Pu3mpONaEumacjW4w6ipPnPw+g2TfywJjeEcpSZaP4Q3YV5HG8D6UjWA4GSkBKculWpdCMadx0usMomsSS/74QgpYqcPkmamB4nVv1JxczYITIqItIKjD35IGKAUwAA==".parse().unwrap());

        headers.insert("client-version", "2.0.0".parse().unwrap());

        headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 Edg/123.0.0.0".parse().unwrap());

        WxOpenapiClient {
            cli: reqwest::Client::builder().default_headers(headers).cookie_provider(Arc::clone(&cookie_store)).build().unwrap(),
            cookie_store,
            ticket_re: Regex::new(r#"ticket=(\w*)@"#).unwrap(),
            login: Login::new(),
            base: Base::new(),
            member: Member::new(),
            message: Message::new(),
        }
    }
    pub async fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.login.get_qr_code_uuid(&self.cli).await?;
        let _ = self.login.get_qr_code(&self.cli).await?;
        println!("check login");
        while !self.login.check(&self.cli).await? {
            time::sleep(Duration::from_secs(1)).await;
            println!("check login again");
        }
        self.base.init(&self.cli, &self.cookie_store, &self.login.redirect_uri).await?;
        self.member.init(&self.cli, &self.base).await?;

        Ok(())
    }

    pub async fn run(&mut self) {
        let _ = self.message.send_text_msg(&self.cli, &self.base, &self.base.user.user_name,
                                           "@c28883f062e73285f22190312d220bb1a45494c8ed99a9fde001179e05319abe",
                                           "").await;
        self.message.listen(&self.cli, &mut self.base).await;
    }
}