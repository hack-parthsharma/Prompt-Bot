use std::convert::Infallible;
use std::time::Duration;

use teloxide::{Bot, payloads};
use teloxide::dispatching::update_listeners::{self, AsUpdateStream};
use teloxide::prelude::{Request, StreamExt};
use teloxide::requests::JsonRequest;
use teloxide::types::{
    AllowedUpdate, InlineKeyboardButton, InlineKeyboardButtonKind, ParseMode, ReplyMarkup,
    UpdateKind,
};

use crate::share::{BUTTON_SEP, Result, ROW_SEP};

type SendMessage = JsonRequest<payloads::SendMessage>;
type EditMessageText = JsonRequest<payloads::EditMessageText>;

pub async fn find_chat_id(token: impl Into<String>) -> Result<Infallible> {
    let mut update_listener = update_listeners::polling(
        Bot::new(token),
        Some(Duration::from_secs(10)),
        None,
        Some(vec![AllowedUpdate::MyChatMember]),
    );
    let mut stream = Box::pin(update_listener.as_stream());
    while let Some(update) = stream.next().await {
        let update = update?;
        match update.kind {
            UpdateKind::MyChatMember(v) => println!("{}", v.chat.id),
            _ => continue,
        };
    }
    panic!("never happen");
}

pub async fn get_prompt(
    token: impl Into<String>,
    chat_id: i64,
    text: impl AsRef<str>,
    keyboard: impl AsRef<str>,
) -> Result<String> {
    let text = text.as_ref();
    let bot = Bot::new(token);
    let msg_out = SendMessage::new(
        bot.clone(),
        payloads::SendMessage {
            chat_id: chat_id.into(),
            text: text.into(),
            parse_mode: None,
            entities: None,
            disable_web_page_preview: None,
            disable_notification: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: Some(ReplyMarkup::inline_kb(
                keyboard
                    .as_ref()
                    .split(ROW_SEP)
                    .map(|s| {
                        s.split(BUTTON_SEP)
                            .map(|s| InlineKeyboardButton {
                                text: s.into(),
                                kind: InlineKeyboardButtonKind::CallbackData(s.into()),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )),
        },
    )
    .send()
    .await?;
    let mut update_listener = update_listeners::polling(
        bot.clone(),
        Some(Duration::from_secs(10)),
        None,
        Some(vec![AllowedUpdate::CallbackQuery]),
    );
    let mut stream = Box::pin(update_listener.as_stream());
    let mut res = String::new();
    while let Some(update) = stream.next().await {
        let update = update?;
        let query = match update.kind {
            UpdateKind::CallbackQuery(v) => v,
            _ => continue,
        };
        let msg = match query.message {
            Some(v) => v,
            None => continue,
        };
        if msg_out.id != msg.id || msg.chat_id() != chat_id {
            continue;
        }
        let data = match query.data {
            Some(s) => s,
            None => continue,
        };
        EditMessageText::new(
            bot.clone(),
            payloads::EditMessageText {
                chat_id: chat_id.into(),
                message_id: msg.id,
                text: format!("{}\n*{}*", text, &data),
                parse_mode: Some(ParseMode::MarkdownV2),
                entities: None,
                disable_web_page_preview: None,
                reply_markup: None,
            },
        )
        .send()
        .await?;
        res = data;
        break;
    }
    Ok(res)
}
