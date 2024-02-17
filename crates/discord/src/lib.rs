use dotenvy_macro::dotenv as var;
use serenity::all::{Command, GuildId, Interaction, Ready};
use serenity::async_trait;
use serenity::builder::{
    CreateAutocompleteResponse, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::prelude::*;
use sqlx::PgPool;

mod commands;

const DISCORD_BOT_TOKEN: &str = var!("DISCORD_BOT_TOKEN");
pub const DISCORD_BOT_ID: &str = var!("DISCORD_BOT_ID");
pub const DISCORD_BOT_GUILD_ID: &str = var!("DISCORD_BOT_GUILD_ID");

pub struct PoolData;
impl TypeMapKey for PoolData {
    type Value = PgPool;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let pool = {
            let lock = ctx.data.read().await;
            lock.get::<PoolData>().unwrap().clone()
        };
        if let Interaction::Autocomplete(command) = interaction {
            println!("command = '{}'", command.data.name.as_str());
            let choices = match command.data.name.as_str() {
                "whereis" => {
                    Some(commands::whereis::run_autocomplete(&pool, &command.data.options()).await)
                }
                _ => None,
            };
            println!("choices1 = {choices:?}");
            let choices = choices.unwrap_or(vec![]);
            println!("choices2 = {choices:?}");
            let data = CreateAutocompleteResponse::new().set_choices(choices);
            let builder = CreateInteractionResponse::Autocomplete(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to autocomplete request: {err}")
            }
        } else if let Interaction::Command(command) = interaction {
            println!("Got command: {command:#?}");

            let content = match command.data.name.as_str() {
                "test" => Some(commands::test::run(&command.data.options())),
                "gentoken" => Some(commands::api_key::run(&command.data.options())),
                "whereis" => Some(commands::whereis::run(&pool, &command.data.options()).await),
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(err) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {err}");
                }
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            DISCORD_BOT_GUILD_ID
                .parse()
                .expect("DISCORD_GUILD_ID must be an integer"),
        );

        let ssi_commands = guild_id
            .set_commands(&ctx.http, vec![commands::api_key::register()])
            .await;
        println!("SSI commands registered: {:#?}", ssi_commands);

        let global_commands = Command::set_global_commands(
            &ctx.http,
            vec![commands::test::register(), commands::whereis::register()],
        )
        .await;
        println!("Global commands registered: {:#?}", global_commands);
    }
}

pub async fn run_bot(pool: &PgPool) {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(DISCORD_BOT_TOKEN, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let pool = pool.clone();
        let mut lock = client.data.write().await;
        lock.insert::<PoolData>(pool);
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
