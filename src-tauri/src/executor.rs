use crate::openapi::base::Base;
use crate::openapi::message::MessageService;
use crate::openapi::rule::{AutoReplyRuleService, OpenAiMessage, ScheduledRuleService};
use reqwest::Client;
use std::error::Error;

pub struct Executor {
    message_service: MessageService,
    auto_reply_rule_service: AutoReplyRuleService,
    scheduled_rule_service: ScheduledRuleService,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            message_service: MessageService::new(),
            auto_reply_rule_service: AutoReplyRuleService::new(),
            scheduled_rule_service: ScheduledRuleService::new(),
        }
    }

    pub async fn auto_reply(
        &mut self,
        cli: Client,
        base: &mut Base,
    ) -> Result<bool, Box<dyn Error>> {
        let code = self.message_service.check(cli.clone(), base).await?;
        match code.unwrap().as_str() {
            "1102" => {
                self.message_service.alive = false;
                return Ok(false);
            }
            _ => {}
        }

        let sync_response = self.message_service.sync(cli.clone(), base).await?;

        println!("sync_response={:?}", sync_response);

        base.sync_key = sync_response.sync_check_key;

        let rules = self.auto_reply_rule_service.m_get_running().unwrap();

        for msg in sync_response.add_msg_list.iter() {
            println!("msg={}", serde_json::to_string(msg).unwrap());

            match msg.msg_type {
                1 => {
                    if msg.to_user_name != base.user.user_name {
                        continue;
                    }

                    for rule in rules.iter() {
                        if !rule.group.hit(&msg.from_user_name) {
                            continue;
                        }

                        for reply in rule.reply.iter() {
                            if !reply.hit(&msg.content) {
                                continue;
                            }

                            let messages = self.message_service.history.get(&msg.from_user_name);

                            let mut input = messages.clone();

                            input.push(OpenAiMessage {
                                role: "user".to_string(),
                                content: msg.content.to_string(),
                            });

                            let output = reply.content(&input).await?;
                            if output.content.is_empty() {
                                continue;
                            }
                            println!("reply={:?}", &output);
                            self.message_service
                                .send_text_msg(
                                    cli.clone(),
                                    base,
                                    &msg.to_user_name,
                                    vec![&msg.from_user_name, "filehelper"],
                                    output.content.as_str(),
                                )
                                .await?;

                            self.message_service.history.push(
                                msg.from_user_name.as_str(),
                                vec![
                                    OpenAiMessage {
                                        role: "user".to_string(),
                                        content: msg.content.to_string(),
                                    },
                                    output,
                                ],
                            );
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(true)
    }
}
