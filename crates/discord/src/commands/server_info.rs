use crate::{ansi::mc_to_ansi, sanitize, Template, EMBED_COLOR_ERROR};
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
                            "Failed to parse `server` as SocketV4Addr\n\n`{addr:?}`"
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
            let motd = match &server.description {
                Some(description) => description
                    .lines()
                    .take(2)
                    .map(|line| {
                        let line = sanitize(line.trim_matches([' ', '\t']));
                        mc_to_ansi(line)
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                None => "No MOTD".to_string(),
            };
            let embed = CreateEmbed::template()
                .title(format!("{}:{}", server.ip(), server.port()))
                .url(format!("https://{}/server/{id}", config::get().web.domain))
                .image(format!(
                    "https://{}/server/{id}/favicon.png",
                    config::get().web.domain
                ))
                .description(format!("```ansi\n{}\n```", motd))
                .field(
                    "Server Version",
                    format!(
                        "{} - {}",
                        sanitize(server.version_name.unwrap_or("Unknown".into())),
                        server.version_protocol.unwrap_or(-1)
                    ),
                    true,
                )
                .field(
                    "Players",
                    format!(
                        "{} / {}",
                        server.online_players.unwrap_or(-1),
                        server.max_players.unwrap_or(-1)
                    ),
                    true,
                )
                .field('\t', '\t', false)
                .field("Discovered", format!("<t:{}:R>", server.discovered), true)
                .field("Last Seen", format!("<t:{}:R>", server.last_seen), true)
                .field('\t', '\t', false)
                .field("Online Mode", "todo!()", true)
                .field("Whitelisted", "todo!()", true)
                .footer(CreateEmbedFooter::new(format!("Query took {duration:?}")));
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed))
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::template()
                        .color(EMBED_COLOR_ERROR)
                        .title("Server Not Found")
                        .description("The provided server was not found in the database.\nThe server either does not exist or the scanner has not found it."),
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
