use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use reqwest::header::HeaderMap;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use reqwest::RequestBuilder;

#[warn(dead_code)]
pub struct WxOpenapiClient {
    pub cli: Client,
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

        let client = Client::builder()
            .default_headers(headers)
            .cookie_provider(cookie_store.clone())
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(15))
            // .tcp_keepalive(Some(Duration::from_secs(30)))
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap();

        WxOpenapiClient { cli: client }
    }

    pub fn client(&self) -> Client {
        self.cli.clone()
    }

    pub async fn request_with_retry<T, F, Fut>(&self, f: F) -> reqwest::Result<T> 
    where
        F: Fn(Client) -> Fut,
        Fut: std::future::Future<Output = reqwest::Result<T>>,
    {
        let mut retries = 0;
        let max_retries = 3;
        let mut delay = 1;

        loop {
            match f(self.client()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if retries >= max_retries {
                        return Err(e);
                    }
                    println!("请求失败，正在重试 {}/{}...", retries + 1, max_retries);
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    retries += 1;
                    delay *= 2;
                }
            }
        }
    }
}

impl WxOpenapiClient {
    pub async fn get(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        self.request_with_retry(|client| client.get(url).send()).await
    }

    pub async fn post(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        self.request_with_retry(|client| client.post(url).send()).await
    }
}
