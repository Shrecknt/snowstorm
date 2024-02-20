use crate::{Template, EMBED_COLOR_ERROR};
use dotenvy_macro::dotenv as var;
use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
};
use sqlx::PgPool;
use std::{net::SocketAddrV4, str::FromStr, time::Instant};

pub async fn run(pool: &PgPool, options: &[ResolvedOption<'_>]) -> CreateInteractionResponse {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(server),
        ..
    }) = options.first()
    {
        let addr = SocketAddrV4::from_str(server);
        let Ok(addr) = addr else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::template()
                        .color(EMBED_COLOR_ERROR)
                        .title("Invalid Argument")
                        .description(format!(
                            "Failed to parse string as SocketV4Addr\n\n`{addr:?}`"
                        )),
                ),
            );
        };
        let (ip, port) = (addr.ip(), addr.port());
        let start_time = Instant::now();
        let server = database::server::PingResult::from_ip_port(ip, port, pool).await;
        let end_time = Instant::now();
        let duration = end_time - start_time;
        if let Some(server) = server {
            let id = server.id.unwrap();
            let embed = CreateEmbed::template()
                .title(format!("{}:{}", server.ip(), server.port()))
                .url(format!("{}/server/{id}", var!("BASE_URI")))
                .description(format!(
                    "{}\ndiscovered = {}, last seen = {}",
                    server.description.unwrap_or("No description".to_string()),
                    server.discovered,
                    server.last_seen
                ))
                .footer(CreateEmbedFooter::new(format!(
                    "Query took {duration:?} \u{2022} Found {} servers",
                    "todo!()"
                )));
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed))
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::template()
                        .color(EMBED_COLOR_ERROR)
                        .description("use autocomplete u goober"),
                ),
            )
        }
    } else {
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().embed(
                CreateEmbed::template()
                    .color(EMBED_COLOR_ERROR)
                    .description("something weird happen"),
            ),
        )
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("servers")
        .description("Find servers with various filters")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "server",
                "The server to look for",
            )
            .required(true),
        )
}
