use crate::Template;
use serenity::{
    all::ResolvedOption,
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
    },
};

pub fn run(_options: &[ResolvedOption]) -> CreateInteractionResponse {
    let token = "".to_string();
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .embed(CreateEmbed::template().description(format!("Generated token {token}"))),
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("api_key")
        .dm_permission(false)
        .description("Generate an API key")
}

pub fn register_alias() -> CreateCommand {
    CreateCommand::new("gentoken")
        .dm_permission(false)
        .description("Generate an API key (alias of /api_key)")
}
