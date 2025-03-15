use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct WxChatError {
    message: String,
}

impl fmt::Display for WxChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WxChatError: {}", self.message)
    }
}

impl Error for WxChatError {}

impl WxChatError {
    pub fn new(message: &str) -> Self {
        WxChatError {
            message: message.to_string(),
        }
    }
}
