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

#[poise::command(slash_command, prefix_command)]
async fn hello(
    ctx: Context<'_>,
    #[description = "User to tag"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("Hello <@{:?}>!", u.id.get());
    let builder = CreateReply::default().content(response);
    ctx.send(builder).await?;

    Ok(())
}

#[derive(Deserialize)]
struct WorldTime {
    timezone: String,
    datetime: String,
}

impl WorldTime {
    pub fn to_timestamp(self) -> Timestamp {
        let t = self.datetime.split(' ')
            .collect::<Vec<&str>>()
            .join("T") + ".000Z";

        Timestamp::from_str(&t).unwrap()
    }
}

#[poise::command(slash_command, prefix_command)]
async fn timezone(
    ctx: Context<'_>,
    #[description = "City"] city: String,
) -> Result<(), Error> {

    let ninja_api_key = std::env::var("API_NINJAS_KEY").expect("missing API_NINJAS_KEY");

    let client = reqwest::Client::new();
    let res = client
        .get(format!("https://api.api-ninjas.com/v1/worldtime?city={}", city))
        .header("X-Api-Key", ninja_api_key)
        .send()
        .await?
        .json::<WorldTime>()
        .await
        .unwrap();

    let embed = CreateEmbed::new()
        .title(format!("City: {}", city))
        .description(format!("Timezone: {}", res.timezone))
        .timestamp(res.to_timestamp());
    let builder = CreateReply::default().embed(embed);
    ctx.send(builder).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn join(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let u = ctx.author();
    let channel_id = ctx.guild().unwrap().voice_states.get(&u.id).unwrap().channel_id.unwrap();
    // let channel = ctx.guild().unwrap().channels.get(&channel_id).unwrap();
    let guild_id = ctx.guild().unwrap().id;

    let manager = songbird::get(&ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    ctx.say(format!("Joining <#{}>!", channel_id)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn leave(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let u = ctx.author();
    let channel_id = ctx.guild().unwrap().voice_states.get(&u.id).unwrap().channel_id.unwrap();
    let guild_id = ctx.guild().unwrap().id;

    let manager = songbird::get(&ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();


    manager.remove(guild_id).await.expect("TODO: panic message");

    ctx.say(format!("Leaving <#{}>!", channel_id)).await?;
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