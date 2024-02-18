use serenity::{
    all::ResolvedOption,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
};

pub fn run(_options: &[ResolvedOption]) -> CreateInteractionResponse {
    let token = "".to_string();
    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content(format!("Generated token {token}")),
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("gentoken")
        .dm_permission(false)
        .description("Generate an API key")
}
