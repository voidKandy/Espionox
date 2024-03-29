use core::time::Duration;
use std::collections::HashMap;

use espionox::{
    agents::{
        independent::IndependentAgent,
        memory::{messages::MessageRole, Message},
        Agent,
    },
    environment::{
        dispatch::{
            listeners::ListenerMethodReturn, Dispatch, EnvListener, EnvMessage, EnvNotification,
        },
        Environment,
    },
    language_models::{ModelProvider, LLM},
};

#[derive(Debug)]
pub struct SummarizeAtLimit {
    limit: usize,
    summarizer: IndependentAgent,
    watched_agent_id: String,
}

impl SummarizeAtLimit {
    fn new(limit: usize, watched_agent_id: &str, summarizer: IndependentAgent) -> Self {
        Self {
            limit,
            watched_agent_id: watched_agent_id.to_owned(),
            summarizer,
        }
    }
}

impl EnvListener for SummarizeAtLimit {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        if let EnvMessage::Response(noti) = env_message {
            if let EnvNotification::AgentStateUpdate {
                agent_id, cache, ..
            } = noti
            {
                if cache.ref_filter_by(MessageRole::System, false).len() >= self.limit
                    && agent_id == &self.watched_agent_id
                {
                    return Some(env_message);
                }
            }
        }
        None
    }

    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            let cache_to_summarize = match trigger_message {
                EnvMessage::Response(ref noti) => match noti {
                    EnvNotification::AgentStateUpdate { cache, .. } => cache.to_string(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            let message = Message::new_user(&format!(
                "Summarize this chat history: {}",
                cache_to_summarize
            ));
            self.summarizer.mutate_agent_cache(|c| c.push(message));

            let summary = self.summarizer.io_completion().await?;

            let watched_agent = dispatch
                .get_agent_mut(&self.watched_agent_id)
                .expect("Failed to get watched agent");
            watched_agent.cache.mut_filter_by(MessageRole::System, true);
            watched_agent.cache.push(Message::new_assistant(&summary));
            Ok(trigger_message)
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_KEY").unwrap();
    let mut map = HashMap::new();
    map.insert(ModelProvider::Anthropic, api_key);
    let mut env = Environment::new(Some("testing"), map);
    let agent = Agent::new(Some("You are jerry!!"), LLM::default_anthropic());
    let mut jerry_handle = env.insert_agent(Some("jerry"), agent).await.unwrap();

    let summarizer = env
        .make_agent_independent(Agent::new(
            Some("Your job is to summarize chunks of a conversation"),
            LLM::default_anthropic(),
        ))
        .await
        .unwrap();
    let sal = SummarizeAtLimit::new(5usize, "jerry", summarizer);

    env.insert_listener(sal).await.unwrap();
    let mut env_handle = env.spawn_handle().unwrap();
    let message = Message::new_user("im saying things to fill space");

    for _ in 0..=5 {
        jerry_handle
            .request_cache_push(message.clone())
            .await
            .expect("failed to request cache push");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let mut stack = env_handle.finish_current_job().await.unwrap();
    let latest = stack.pop_back().unwrap();

    // env.finalize_dispatch().await.unwrap();
    if let EnvNotification::AgentStateUpdate { cache, .. } = latest {
        println!("STACK: {:?}", cache);
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.as_ref()[0].role, MessageRole::System);
        assert_eq!(cache.as_ref()[1].role, MessageRole::Assistant);
        assert_eq!(cache.as_ref()[2].role, MessageRole::User);
        println!("All asserts passed, summarize at limit working as expected");
        return;
    }
    println!("Incorrect notification in last place: {:?}", latest);
}
