use crate::{Template, EMBED_COLOR_ERROR, NUM_CODES};
use database::autocomplete::AutocompleteResults;
use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{
        AutocompleteChoice, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
};
use sqlx::PgPool;
use std::time::Instant;

pub async fn run(pool: &PgPool, options: &[ResolvedOption<'_>]) -> CreateInteractionResponse {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(username_uuid),
        ..
    }) = options.first()
    {
        let id = username_uuid.parse();
        let Ok(id) = id else {
            return CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::template()
                        .color(EMBED_COLOR_ERROR)
                        .title("Invalid Argument")
                        .description(
                            format!("Invalid value for `username_uuid`\nTry using the autocomplete menu\n\n`{id:?}`"),
                        ),
                ),
            );
        };
        let start_time = Instant::now();
        let player = database::player::PlayerInfo::from_id(id, pool).await;
        if let Some(player) = player {
            let servers = database::server::PingResult::from_player_id(id, pool).await;
            let end_time = Instant::now();
            let duration = end_time - start_time;
            let display_servers = servers
                .iter()
                .take(8)
                .enumerate()
                .map(|(index, server)| {
                    format!(
                        "{} `{}:{}`",
                        NUM_CODES[index + 1],
                        server.ip(),
                        server.port()
                    )
                })
                .collect::<Vec<_>>();
            let embed = CreateEmbed::template()
                .title(player.username)
                .url(format!("https://snowstorm.shrecked.dev/player/{id}"))
                .description(format!("{}\n{}", player.uuid, display_servers.join("\n")))
                .footer(CreateEmbedFooter::new(format!("Query took {duration:?}")));
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed))
        } else {
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(
                    CreateEmbed::template()
                        .color(EMBED_COLOR_ERROR)
                        .title("Invalid Argument")
                        .description(
                            "Invalid value for `username_uuid`\nTry using the autocomplete menu\n\n`Provided index not found in database`",
                        ),
                ),
            )
        }
    } else {
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(
            CreateEmbed::template()
                .color(EMBED_COLOR_ERROR)
                .title("Missing Argument")
                .description("`username_uuid` argument not found.\nThis shouldn't be possible :/\nthanks discor"),
        ))
    }
}

pub async fn run_autocomplete(
    pool: &PgPool,
    options: &[ResolvedOption<'_>],
) -> Vec<AutocompleteChoice> {
    if let Some(ResolvedOption {
        value:
            ResolvedValue::Autocomplete {
                kind: _,
                value: username_uuid,
            },
        ..
    }) = options.first()
    {
        let results =
            database::player::PlayerInfo::autocomplete_username(username_uuid, pool).await;
        if let AutocompleteResults::Username { players } = results {
            let mut res = Vec::with_capacity(players.len());
            for (id, uuid, username) in players {
                let display_text = format!("{username} - {uuid}");
                if !(1..100).contains(&display_text.len()) {
                    continue;
                }
                let autocomplete_result = AutocompleteChoice::new(display_text, id.to_string());
                res.push(autocomplete_result);
            }
            return res;
        }
        vec![]
    } else {
        vec![]
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("where_is")
        .description("Find a player by username or UUID")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "username_uuid",
                "The player to look for",
            )
            .required(true)
            .set_autocomplete(true),
        )
}
