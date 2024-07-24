use std::collections::{HashMap, HashSet};
use serde::Deserialize;

pub trait Rule {
    fn hit(&self, user_name: &str) -> bool;

    fn reply(&self, message: &str) -> Option<String>;

    fn add_member(&mut self, user_name: &str);

    fn remove_member(&mut self, user_name: &str);
}

pub enum RuleType {
    Text
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum ReplyType {
    Default,
    Match,
    Ai,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextMessageRule {
    member_group: HashSet<String>,
    reply_type: ReplyType,
    default_reply_message: String,
    match_reply_message: HashMap<String, String>,
}


impl Rule for TextMessageRule {
    fn hit(&self, user_name: &str) -> bool {
        self.member_group.contains(&user_name.to_string())
    }

    fn reply(&self, message: &str) -> Option<String> {
        match self.reply_type {
            ReplyType::Default => Some(self.default_reply_message.clone()),
            ReplyType::Match => None,
            _ => None
        }
    }

    fn add_member(&mut self, user_name: &str) {
        self.member_group.insert(user_name.to_string());
    }

    fn remove_member(&mut self, user_name: &str) {
        self.member_group.remove(user_name);
    }
}