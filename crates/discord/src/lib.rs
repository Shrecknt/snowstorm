use dotenvy_macro::dotenv as var;
use serenity::all::{Command, GuildId, Interaction, Ready};
use serenity::async_trait;
use serenity::builder::{
    CreateAutocompleteResponse, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::model::Color;
use serenity::prelude::*;
use sqlx::PgPool;

mod commands;

pub mod ansi;

pub const NUM_CODES: [&str; 10] = [
    ":zero:", ":one:", ":two:", ":three:", ":four:", ":five:", ":six:", ":seven:", ":eight:",
    ":nine:",
];

const DISCORD_BOT_TOKEN: &str = var!("DISCORD_BOT_TOKEN");
pub const DISCORD_BOT_ID: &str = var!("DISCORD_BOT_ID");
pub const DISCORD_BOT_GUILD_ID: &str = var!("DISCORD_BOT_GUILD_ID");

pub const EMBED_COLOR: Color = Color::from_rgb(30, 110, 220);
pub const EMBED_COLOR_ERROR: Color = Color::from_rgb(250, 70, 70);

trait Template {
    fn template() -> Self;
}
impl Template for CreateEmbed {
    fn template() -> Self {
        Self::new().color(EMBED_COLOR)
    }
}

pub fn sanitize<T: ToString>(content: T) -> String {
    content.to_string()
}

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
            let choices = match command.data.name.as_str() {
                "where_is" => {
                    Some(commands::where_is::run_autocomplete(&pool, &command.data.options()).await)
                }
                _ => None,
            };
            let choices = choices.unwrap_or(vec![]);
            let data = CreateAutocompleteResponse::new().set_choices(choices);
            let builder = CreateInteractionResponse::Autocomplete(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to autocomplete request: {err}")
            }
        } else if let Interaction::Command(command) = interaction {
            let content: Option<CreateInteractionResponse> = match command.data.name.as_str() {
                "test" => Some(commands::test::run(&command.data.options())),
                "api_key" => Some(commands::api_key::run(&command.data.options())),
                "gentoken" => Some(commands::api_key::run(&command.data.options())),
                "where_is" => Some(commands::where_is::run(&pool, &command.data.options()).await),
                "server_info" => {
                    Some(commands::server_info::run(&pool, &command.data.options()).await)
                }
                "servers" => Some(commands::servers::run(&pool, &command.data.options()).await),
                _ => {
                    let data =
                        CreateInteractionResponseMessage::new().content("not implemented :(");
                    let builder = CreateInteractionResponse::Message(data);
                    Some(builder)
                }
            };

            if let Some(content) = content {
                if let Err(err) = command.create_response(&ctx.http, content).await {
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

        let _ssi_commands = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    commands::api_key::register(),
                    commands::api_key::register_alias(),
                ],
            )
            .await
            .unwrap();

        let _global_commands = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::test::register(),
                commands::where_is::register(),
                commands::server_info::register(),
                commands::servers::register(),
            ],
        )
        .await
        .unwrap();

        println!("Loaded application commands!");
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
