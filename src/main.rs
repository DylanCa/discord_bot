mod commands;
use commands::{register_commands, Data};

use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use serenity::GuildId;

use reqwest::Client as HttpClient;
use serenity::prelude::TypeMapKey;
use songbird::SerenityInit;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let guild_id = GuildId::new(
        std::env::var("GUILD_ID")
            .expect("missing GUILD_ID")
            .parse::<u64>()
            .unwrap(),
    );
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: register_commands(),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                    .await?;
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

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}
