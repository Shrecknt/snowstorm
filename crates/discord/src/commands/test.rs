use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{CreateCommand, CreateCommandOption},
};

pub fn run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    {
        format!("{}'s id is {}", user.tag(), user.id)
    } else {
        "Please provide a valid user".to_string()
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("test")
        .description("test a thing")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "id", "le user").required(true),
        )
}
