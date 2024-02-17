use serenity::{all::ResolvedOption, builder::CreateCommand};

pub fn run(_options: &[ResolvedOption]) -> String {
    let token = "".to_string();
    format!("Generated token {token}")
}

pub fn register() -> CreateCommand {
    CreateCommand::new("gentoken")
        .dm_permission(false)
        .description("Generate an API key")
}
