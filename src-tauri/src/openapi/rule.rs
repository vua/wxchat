use crate::openapi::member::ContactMember;
use crate::openapi::tool::file_tool;
use cron::Schedule;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;
use tokio::io;

pub static APPDATA_PATH: Lazy<Mutex<PathBuf>> = Lazy::new(|| Mutex::new(PathBuf::new()));

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OpenAi {
    pub id: String,
    pub name: String,
    pub source: String,
    pub token: String,
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OpenAiConfig {
    pub source: String,
    pub url: String,
    pub model: Vec<String>,
}

impl OpenAiConfig {
    pub fn new(source: &str, url: &str, model: Vec<&str>) -> Self {
        OpenAiConfig {
            source: source.to_string(),
            url: url.to_string(),
            model: model.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
}

impl OpenAiRequest {
    pub fn new() -> Self {
        OpenAiRequest {
            model: String::new(),
            messages: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub message: OpenAiMessage,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

pub struct OpenAiService {
    data_path: PathBuf,
}

impl OpenAiService {
    pub fn new() -> OpenAiService {
        OpenAiService {
            data_path: APPDATA_PATH.lock().unwrap().join("data/user/openai.json"),
        }
    }
    pub fn create(&mut self, openapi: OpenAi) -> io::Result<Vec<OpenAi>> {
        let mut openai_list = self.m_get()?;
        openai_list.push(openapi);
        self.write(&openai_list)?;
        Ok(openai_list)
    }

    pub fn m_get(&mut self) -> io::Result<Vec<OpenAi>> {
        let mut data = file_tool::get_or_create_file(self.data_path.as_path())?;
        if data.is_empty() {
            data = "[]".to_string();
        }
        let openai_list: Vec<OpenAi> = serde_json::from_str(&data)?;
        Ok(openai_list)
    }

    pub fn get(&mut self, id: &str) -> Result<Option<OpenAi>, Box<dyn Error>> {
        if id.is_empty() {
            return Ok(None);
        }
        let openai_list = self.m_get()?;

        for openai in openai_list {
            if openai.id == id {
                return Ok(Some(openai));
            }
        }
        Err(Box::from("OpenAi not found"))
    }

    pub fn write(&mut self, openai_list: &Vec<OpenAi>) -> io::Result<()> {
        let data = serde_json::to_string(&openai_list)?;
        fs::write(self.data_path.as_path(), data)?;
        Ok(())
    }

    pub fn del(&mut self, id: String) -> io::Result<Vec<OpenAi>> {
        let mut openai_list = self.m_get()?;
        let mut find = false;
        let mut index = 0;
        for (i, openai) in openai_list.iter().enumerate() {
            if openai.id == id {
                index = i;
                find = true;
                break;
            }
        }
        if !find {
            return Ok(openai_list);
        }
        openai_list.remove(index);
        self.write(&openai_list)?;
        Ok(openai_list)
    }

    pub fn update(&mut self, openai: OpenAi) -> io::Result<Vec<OpenAi>> {
        let mut openai_list = self.m_get()?;

        for item in openai_list.iter_mut() {
            if item.id == openai.id {
                *item = openai;
                break;
            }
        }
        self.write(&openai_list)?;
        Ok(openai_list)
    }

    pub async fn test(&mut self, openai: OpenAi) -> Result<String, Box<dyn Error>> {
        let msg = self
            .context_generate(
                openai,
                &vec![OpenAiMessage {
                    role: "user".to_string(),
                    content: "你好".to_string(),
                }],
            )
            .await?;
        Ok(msg.content)
    }

    pub async fn context_generate(
        &mut self,
        openai: OpenAi,
        messages: &Vec<OpenAiMessage>,
    ) -> Result<OpenAiMessage, Box<dyn Error>> {
        let mut url = String::new();
        let configs = OpenAiConfigService::new().m_get()?;
        for config in configs.iter() {
            if config.source == openai.source {
                url = config.url.clone();
                break;
            }
        }
        if url.is_empty() {
            return Err(Box::from("url not found"));
        }

        let mut request = OpenAiRequest::new();
        request.model = openai.model.clone();
        if openai.prompt != "" {
            request.messages.push(OpenAiMessage {
                role: "system".to_string(),
                content: openai.prompt.clone(),
            });
        }

        for message in messages {
            request.messages.push(OpenAiMessage {
                role: message.role.to_string(),
                content: message.content.clone(),
            });
        }

        let body = serde_json::to_string(&request)?;
        let data = Client::new()
            .post(url)
            .header("Authorization", format!("Bearer {}", openai.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        println!("openai_data={}", data);
        let response: OpenAiResponse = serde_json::from_str(&data).unwrap_or(OpenAiResponse {
            choices: vec![Choice {
                message: OpenAiMessage {
                    role: "user".to_string(),
                    content: "测试异常 , 请检测配置".to_string(),
                },
            }],
        });
        let choice = response.choices.get(0).unwrap();
        Ok(OpenAiMessage {
            role: choice.message.role.clone(),
            content: choice.message.content.clone(),
        })
    }
}

pub struct OpenAiConfigService {
    data_path: PathBuf,
}

impl OpenAiConfigService {
    pub fn new() -> OpenAiConfigService {
        OpenAiConfigService {
            data_path: APPDATA_PATH.lock().unwrap().join("data/system/config.json"),
        }
    }

    pub fn m_get(&mut self) -> io::Result<Vec<OpenAiConfig>> {
        let mut data = file_tool::get_or_create_file(self.data_path.as_path())?;
        if data.is_empty() {
            data = "[]".to_string();
        }
        let config: Vec<OpenAiConfig> = serde_json::from_str(&data)?;
        Ok(config)
    }

    pub fn write(&mut self, config: &Vec<OpenAiConfig>) -> io::Result<()> {
        let data = serde_json::to_string(config)?;
        fs::write(self.data_path.as_path(), data)?;
        Ok(())
    }

    pub fn create(&mut self, config: OpenAiConfig) -> io::Result<Vec<OpenAiConfig>> {
        let mut configs = self.m_get()?;
        for item in configs.iter_mut() {
            if item.source == config.source {
                return Ok(configs);
            }
        }
        configs.push(config);
        self.write(&configs)?;
        Ok(configs)
    }

    pub fn update(&mut self, configs: Vec<OpenAiConfig>) -> io::Result<Vec<OpenAiConfig>> {
        self.write(&configs)?;
        Ok(configs)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub members: Vec<ContactMember>,
    pub operator: String,
}

pub struct GroupService {
    data_path: PathBuf,
}

impl GroupService {
    pub fn new() -> GroupService {
        GroupService {
            data_path: APPDATA_PATH.lock().unwrap().join("data/user/groups.json"),
        }
    }

    pub fn create(&mut self, group: Group) -> io::Result<Vec<Group>> {
        let mut groups = self.m_get()?;
        for item in groups.iter_mut() {
            if item.name == group.name {
                return Ok(groups);
            }
        }
        groups.push(group);
        self.write(&groups)?;
        Ok(groups)
    }

    pub fn write(&mut self, groups: &Vec<Group>) -> io::Result<()> {
        let data = serde_json::to_string(&groups)?;
        fs::write(self.data_path.as_path(), data)?;
        Ok(())
    }

    pub fn m_get(&mut self) -> io::Result<Vec<Group>> {
        let mut data = file_tool::get_or_create_file(self.data_path.as_path())?;
        if data.is_empty() {
            data = "[]".to_string();
        }
        let groups: Vec<Group> = serde_json::from_str(&data)?;
        Ok(groups)
    }

    pub fn get(&mut self, id: &str) -> Result<Option<Group>, Box<dyn Error>> {
        if id.is_empty() {
            return Ok(None);
        }
        let groups = self.m_get()?;

        println!("{},{:?}", id, groups);

        for group in groups {
            if group.id == id {
                return Ok(Some(group));
            }
        }
        Err(Box::from("Group not found"))
    }

    pub fn del(&mut self, id: String) -> io::Result<Vec<Group>> {
        let mut groups = self.m_get()?;
        let mut find = false;
        let mut index = 0;
        for (i, group) in groups.iter().enumerate() {
            if group.id == id {
                index = i;
                find = true;
                break;
            }
        }
        if !find {
            return Ok(groups);
        }
        groups.remove(index);
        self.write(&groups)?;
        Ok(groups)
    }

    pub fn update_by_member(&mut self, member_list: &Vec<ContactMember>) -> io::Result<()> {
        let mut groups = self.m_get()?;
        let mut member_map: HashMap<String, &ContactMember> = HashMap::new();

        for group in member_list {
            member_map.insert(group.p_y_quan_pin.clone(), group);
        }
        for group in groups.iter_mut() {
            for member in group.members.iter_mut() {
                let m = member_map.get(&member.p_y_quan_pin).unwrap();
                member.user_name = m.user_name.clone();
            }
        }
        self.write(&groups)?;
        Ok(())
    }
}

impl Group {
    pub fn hit(&self, user_name: &str) -> bool {
        if self.members.len() == 0 {
            match self.id.as_str() {
                "all" => return true,
                "all_membership" => return !user_name.starts_with("@@"),
                "all_account" => return !user_name.starts_with("@@"),
                "all_classroom" => return user_name.starts_with("@@"),
                _ => {}
            }
        }
        for member in &self.members {
            if member.user_name == user_name {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AutoReplyRule {
    pub id: String,
    pub name: String,
    pub group: Group,
    pub reply: Vec<Reply>,
    pub status: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ScheduledRule {
    pub id: String,
    pub cron: String,
    pub name: String,
    pub group: Group,
    pub reply: Vec<Reply>,
    pub status: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Reply {
    pub reply_type: String,
    pub template: Template,
    pub open_ai: OpenAi,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
    pub keywords: Vec<String>,
    pub content: String,
}

impl Reply {
    pub fn hit(&self, content: &str) -> bool {
        if self.reply_type == "AI" {
            return true;
        }
        if self.template.keywords.len() == 0 {
            return true;
        }
        self.template.keywords.iter().any(|k| content.contains(k))
    }

    pub async fn content(
        &self,
        content: &Vec<OpenAiMessage>,
    ) -> Result<OpenAiMessage, Box<dyn Error>> {
        if self.reply_type == "Template" {
            return Ok(OpenAiMessage {
                role: "assistant".to_string(),
                content: self.template.content.clone(),
            });
        }
        let mut service = OpenAiService::new();
        let openai = service.get(&self.open_ai.id)?.unwrap();
        service.context_generate(openai, content).await
    }
}

pub struct AutoReplyRuleService {
    data_path: PathBuf,
}

impl AutoReplyRuleService {
    pub fn new() -> AutoReplyRuleService {
        AutoReplyRuleService {
            data_path: APPDATA_PATH
                .lock()
                .unwrap()
                .join("data/user/auto_reply_rules.json"),
        }
    }

    pub fn create(&mut self, rule: AutoReplyRule) -> io::Result<Vec<AutoReplyRule>> {
        let mut rules = self.m_get()?;
        rules.push(rule);
        self.write(&rules)?;
        Ok(rules)
    }

    pub fn write(&mut self, rules: &Vec<AutoReplyRule>) -> io::Result<()> {
        let data = serde_json::to_string(&rules)?;
        fs::write(self.data_path.as_path(), data)?;
        Ok(())
    }

    pub fn m_get(&mut self) -> io::Result<Vec<AutoReplyRule>> {
        let mut data = file_tool::get_or_create_file(self.data_path.as_path())?;
        if data.is_empty() {
            data = "[]".to_string();
        }
        let rules: Vec<AutoReplyRule> = serde_json::from_str(&data)?;
        Ok(rules)
    }

    pub fn m_get_running(&mut self) -> io::Result<Vec<AutoReplyRule>> {
        let mut rules = self.m_get()?;
        let mut group_service = GroupService::new();
        for rule in rules.iter_mut() {
            if !rule.status {
                continue;
            }
            let group = group_service.get(&rule.group.id).unwrap().unwrap();
            rule.group = group;
        }
        Ok(rules)
    }

    pub fn del(&mut self, id: String) -> io::Result<Vec<AutoReplyRule>> {
        let mut rules = self.m_get()?;
        let mut find = false;
        let mut index = 0;
        for (i, rule) in rules.iter().enumerate() {
            if rule.id == id {
                index = i;
                find = true;
                break;
            }
        }
        if !find {
            return Ok(rules);
        }
        rules.remove(index);
        self.write(&rules)?;
        Ok(rules)
    }

    pub fn update(&mut self, rule: AutoReplyRule) -> io::Result<Vec<AutoReplyRule>> {
        let mut rules = self.m_get()?;

        for item in rules.iter_mut() {
            if item.id == rule.id {
                *item = rule;
                break;
            }
        }
        self.write(&rules)?;
        Ok(rules)
    }
}

pub struct ScheduledRuleService {
    data_path: PathBuf,
}

impl ScheduledRuleService {
    pub fn new() -> ScheduledRuleService {
        ScheduledRuleService {
            data_path: APPDATA_PATH
                .lock()
                .unwrap()
                .join("data/user/scheduled_rules.json"),
        }
    }

    pub fn create(&mut self, rule: ScheduledRule) -> io::Result<Vec<ScheduledRule>> {
        let mut rules = self.m_get()?;
        rules.push(rule);
        self.write(&rules)?;
        Ok(rules)
    }

    pub fn write(&mut self, rules: &Vec<ScheduledRule>) -> io::Result<()> {
        let data = serde_json::to_string(&rules)?;
        fs::write(self.data_path.as_path(), data)?;
        Ok(())
    }

    pub fn m_get(&mut self) -> io::Result<Vec<ScheduledRule>> {
        let mut data = file_tool::get_or_create_file(self.data_path.as_path())?;
        if data.is_empty() {
            data = "[]".to_string();
        }
        let rules: Vec<ScheduledRule> = serde_json::from_str(&data)?;
        Ok(rules)
    }

    pub fn m_get_running(&mut self) -> io::Result<Vec<ScheduledRule>> {
        let mut rules = self.m_get()?;
        let mut group_service = GroupService::new();
        for rule in rules.iter_mut() {
            if !rule.status {
                continue;
            }
            let group = group_service.get(&rule.group.id).unwrap().unwrap();
            rule.group = group;
        }
        Ok(rules)
    }

    pub fn del(&mut self, id: String) -> io::Result<Vec<ScheduledRule>> {
        let mut rules = self.m_get()?;
        let mut find = false;
        let mut index = 0;
        for (i, rule) in rules.iter().enumerate() {
            if rule.id == id {
                index = i;
                find = true;
                break;
            }
        }
        if !find {
            return Ok(rules);
        }
        rules.remove(index);
        self.write(&rules)?;
        Ok(rules)
    }

    pub fn update(&mut self, rule: ScheduledRule) -> io::Result<Vec<ScheduledRule>> {
        let mut rules = self.m_get()?;

        for item in rules.iter_mut() {
            if item.id == rule.id {
                *item = rule;
                break;
            }
        }
        self.write(&rules)?;
        Ok(rules)
    }

    pub async fn schedule(&mut self, rule: &ScheduledRule) {
        // 解析 Cron 表达式
        let schedule = match Schedule::from_str(&rule.cron) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to parse cron expression for {}: {}", rule.name, e);
                return;
            }
        };

        loop {
            // 获取当前时间
            let now = chrono::Utc::now();
            // 计算下一次任务执行的时间
            if let Some(next_time) = schedule.upcoming(chrono::Utc).next() {
                // 计算距离下一次执行的时间差
                let duration = next_time
                    .signed_duration_since(now)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0));
                // 等待到下一次执行时间
                // 执行定时任务
                println!("{} 定时任务执行于: {}", rule.name, next_time);
            }
        }
    }
}
