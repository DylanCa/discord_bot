use std::str::FromStr;
use dotenv::dotenv;
use poise::{CreateReply, serenity_prelude as serenity};
use poise::serenity_prelude::GuildId;
use serenity::async_trait;
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::SerenityInit;
use reqwest::Client as HttpClient;
use serenity::all::{CreateEmbed, Timestamp};
use serenity::prelude::TypeMapKey;
use serde::Deserialize;
use songbird::input::cached::LengthHint::Time;

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}
struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}
#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let guild_id = GuildId::new(std::env::var("GUILD_ID").expect("missing GUILD_ID").parse::<u64>().unwrap());
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(),
                           hello(),
                           join(),
                           leave(),
                           timezone()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id ).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await;
    client.unwrap().start().await.unwrap();
}