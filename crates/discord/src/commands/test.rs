use crate::Template;
use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
};

pub fn run(options: &[ResolvedOption]) -> CreateInteractionResponse {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    {
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(
            CreateEmbed::template().description(format!("{}'s id is {}", user.tag(), user.id)),
        ))
    } else {
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("Please provide a valid user"),
        )
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("test")
        .description("test a thing")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "id", "le user").required(true),
        )
}
