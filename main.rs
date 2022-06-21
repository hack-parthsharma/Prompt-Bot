use std::env;

use argh::FromArgs;

use prompt_bot::{discord, share, telegram};

const ENV_TOKEN: &'static str = "PROMPT_BOT_TOKEN";

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Network {
    Telegram(TelegramNetwork),
    Discord(DiscordNetwork),
}

#[derive(FromArgs, PartialEq, Debug)]
/// use telegram
#[argh(subcommand, name = "tg")]
struct TelegramNetwork {}

#[derive(FromArgs, PartialEq, Debug)]
/// use discord
#[argh(subcommand, name = "discord")]
struct DiscordNetwork {
    #[argh(option, short = 'a')]
    /// application id
    application_id: u64,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Prompt bot.
struct Opts {
    #[argh(option, short = 't')]
    /// bot token (env: PROMPT_BOT_TOKEN)
    token: Option<String>,

    #[argh(option, short = 'c')]
    /// chat id
    chat_id: i128,

    #[argh(subcommand)]
    network: Network,

    #[argh(option, short = 'm')]
    /// message
    message: String,

    #[argh(option, short = 'k', default = "String::from(\"Yes,No\")")]
    /// inline keyboard, use "," for button delimiter, ":" for row delimiter (default: Yes,No)
    keyboard: String,

    #[argh(switch, short = 's')]
    /// exit success if first button text is pushed else failure
    silent: bool,
}

fn get_opts() -> Opts {
    let mut opts: Opts = argh::from_env();
    if opts.token.is_none() {
        opts.token = Some(env::var(ENV_TOKEN).expect("token not found"));
        env::remove_var(ENV_TOKEN);
    }
    opts
}

#[tokio::main]
async fn main() -> share::Result<()> {
    let opts = &get_opts();
    let answer = match opts.network {
        Network::Telegram(_) => main_telegram(opts).await?,
        Network::Discord(ref v) => main_discord(opts, v).await?,
    };
    if opts.silent {
        if answer
            != opts
                .keyboard
                .split(share::ROW_SEP)
                .map(|s| s.split(share::BUTTON_SEP))
                .flatten()
                .next()
                .unwrap_or_default()
        {
            return Err(answer.into());
        }
    } else {
        println!("{}", answer);
    }
    Ok(())
}

async fn main_telegram(opts: &Opts) -> share::Result<String> {
    if opts.chat_id == 0 {
        telegram::find_chat_id(opts.token.as_ref().unwrap()).await?;
        return Ok("".into());
    }
    Ok(telegram::get_prompt(
        opts.token.as_ref().unwrap(),
        opts.chat_id as i64,
        &opts.message,
        &opts.keyboard,
    )
    .await?)
}

async fn main_discord(opts: &Opts, network: &DiscordNetwork) -> share::Result<String> {
    Ok(discord::get_prompt(
        opts.token.as_ref().unwrap(),
        network.application_id,
        opts.chat_id as u64,
        &opts.message,
        &opts.keyboard,
    )
    .await?)
}
