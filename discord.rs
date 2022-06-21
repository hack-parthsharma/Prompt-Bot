use std::result;
use std::sync::Arc;

use serenity::{
    async_trait,
    builder::CreateComponents,
    client::bridge::gateway::ShardManager,
    model::{
        gateway::Ready,
        id::ChannelId,
        interactions::{InteractionResponseType, message_component::ButtonStyle},
    },
    prelude::*,
};

use crate::share::{BUTTON_SEP, Result, ROW_SEP};

struct Handler {
    chat_id: u64,
    text: String,
    keyboard: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let message = ChannelId(self.chat_id)
            .send_message(&ctx.http, |m| {
                m.content(&self.text)
                    .components(|c| create_components(c, &self.keyboard))
            })
            .await;
        if let Err(err) = message {
            stop(ctx, Err(err.to_string())).await;
            return;
        }
        let interaction = message.unwrap().await_component_interaction(&ctx).await;
        if interaction.is_none() {
            stop(ctx, Err("none interaction".into())).await;
            return;
        }
        let interaction = interaction.unwrap();
        let value = component_id_to_button(&interaction.data.custom_id, &self.keyboard);
        if let Err(err) = interaction
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.content(format!("{}\n{}", &self.text, &value))
                            .components(|c| c)
                    })
            })
            .await
        {
            stop(ctx, Err(err.to_string())).await;
            return;
        };
        stop(ctx, Ok(value)).await;
    }
}

fn component_id_to_button(id: &str, keyboard: &str) -> String {
    for (row_idx, row) in keyboard.split(ROW_SEP).enumerate() {
        for (button_idx, button) in row.split(BUTTON_SEP).enumerate() {
            if id == make_button_id(row_idx, button_idx) {
                return button.into();
            }
        }
    }
    panic!("never happen");
}

fn create_components<'a>(c: &'a mut CreateComponents, keyboard: &str) -> &'a mut CreateComponents {
    for (row_idx, row) in keyboard.split(ROW_SEP).enumerate() {
        c.create_action_row(|r| {
            for (button_idx, button) in row.split(BUTTON_SEP).enumerate() {
                r.create_button(|b| {
                    b.style(ButtonStyle::Primary)
                        .label(button)
                        .custom_id(make_button_id(row_idx, button_idx))
                });
            }
            r
        });
    }
    c
}

fn make_button_id(row_idx: usize, button_idx: usize) -> String {
    format!("id_{}_{}", row_idx, button_idx)
}

async fn stop(ctx: Context, answer: result::Result<String, String>) {
    let data = ctx.data.read().await;
    data.get::<ResponseContainer>().unwrap().lock().await.answer = Some(answer);
    data.get::<ShardManagerContainer>()
        .unwrap()
        .lock()
        .await
        .shutdown_all()
        .await;
}

struct ShardManagerContainer;
struct ResponseContainer;

struct Response {
    answer: Option<result::Result<String, String>>,
}

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for ResponseContainer {
    type Value = Arc<Mutex<Response>>;
}

pub async fn get_prompt(
    token: impl AsRef<str>,
    application_id: u64,
    chat_id: u64,
    text: impl AsRef<str>,
    keyboard: impl AsRef<str>,
) -> Result<String> {
    let mut client = Client::builder(token)
        .application_id(application_id)
        .event_handler(Handler {
            chat_id,
            text: text.as_ref().into(),
            keyboard: keyboard.as_ref().into(),
        })
        .await?;
    let resp = Arc::new(Mutex::new(Response { answer: None }));
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<ResponseContainer>(resp.clone());
    }
    client.start().await?;
    let answer = resp.lock().await.answer.as_ref().unwrap().clone()?;
    Ok(answer)
}
