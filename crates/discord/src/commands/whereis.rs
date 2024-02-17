use database::autocomplete::AutocompleteResults;
use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{AutocompleteChoice, CreateCommand, CreateCommandOption},
};
use sqlx::PgPool;

pub async fn run(pool: &PgPool, options: &[ResolvedOption<'_>]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(username_uuid),
        ..
    }) = options.first()
    {
        let Ok(id) = username_uuid.parse() else {
            return "invalid int, use autocomplete ya goob".to_string();
        };
        let player = database::player::PlayerInfo::from_id(id, pool).await;
        if let Some(player) = player {
            format!("found player {player:?}")
        } else {
            "use autocomplete u goober".to_string()
        }
    } else {
        "something weird happen".to_string()
    }
}

pub async fn run_autocomplete(
    pool: &PgPool,
    options: &[ResolvedOption<'_>],
) -> Vec<AutocompleteChoice> {
    println!("options = {options:?}");
    if let Some(ResolvedOption {
        value:
            ResolvedValue::Autocomplete {
                kind: _,
                value: username_uuid,
            },
        ..
    }) = options.first()
    {
        println!("running autocomplete on {username_uuid}");
        let results =
            database::player::PlayerInfo::autocomplete_username(&username_uuid, pool).await;
        println!("got results {results:?}");
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
            println!("res = {res:?}");
            return res;
        }
        vec![]
    } else {
        vec![]
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("whereis")
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
