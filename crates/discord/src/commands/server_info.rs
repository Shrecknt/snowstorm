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
        let Ok(addr) = SocketAddrV4::from_str(server) else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("invalid int, use autocomplete ya goob"),
            );
        };
        let (ip, port) = (addr.ip(), addr.port());
        let start_time = Instant::now();
        let server = database::server::PingResult::from_ip_port(ip, port, pool).await;
        let end_time = Instant::now();
        let duration = end_time - start_time;
        if let Some(server) = server {
            let id = server.id.unwrap();
            let embed = CreateEmbed::new()
                .title(format!("{}:{}", server.ip(), server.port()))
                .url(format!("https://snowstorm.shrecked.dev/server/{id}"))
                .description(format!(
                    "{}\ndiscovered = {}, last seen = {}",
                    server.description.unwrap_or("No description".to_string()),
                    server.discovered,
                    server.last_seen
                ))
                .footer(CreateEmbedFooter::new(format!("Query took {duration:?}")));
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed))
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("use autocomplete u goober"),
            )
        }
    } else {
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("something weird happen"),
        )
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("server_info")
        .description("Find a server by IP")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "server",
                "The server to look for",
            )
            .required(true),
        )
}
