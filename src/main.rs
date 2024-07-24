mod openapi;

use openapi::client;

#[tokio::main]
async fn main() {
    let mut cli = client::WxOpenapiClient::new();

    cli.init().await.unwrap();
    cli.run().await;
}