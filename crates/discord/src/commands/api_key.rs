use crate::{sanitize, Template, EMBED_COLOR_ERROR};
use database::{discord_user::DiscordUserInfo, user::User};
use dotenvy_macro::dotenv as var;
use jwt::UserSession;
use serenity::{
    all::{ResolvedOption, UserId},
    builder::{
        CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
};
use sqlx::PgPool;

pub async fn run(
    pool: &PgPool,
    discord_user_id: UserId,
    _options: &[ResolvedOption<'_>],
) -> (CreateInteractionResponse, bool) {
    let discord_user_id = discord_user_id.to_string();
    let user = DiscordUserInfo::get_discord_id(&discord_user_id, pool).await;
    let Some(user) = user else {
        return (CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().embed(
                CreateEmbed::template()
                    .color(EMBED_COLOR_ERROR)
                    .title("Silly Goose")
                    .description(
                        format!("You can't generate a token without first being registered\nGo to {}/signup to register\nAnd sign up with Discord so the bot can actually know who you are kthxbye", var!("BASE_URI")),
                    ),
            ),
        ), false);
    };
    let Some(user_id) = user.user_id else {
        return (
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(
                CreateEmbed::template()
                    .color(EMBED_COLOR_ERROR)
                    .title("Silly Goose")
                    .description(
                        "Uhh I think you forgor to link your discor account to a Snowstorm account",
                    ),
            )),
            false,
        );
    };
    let session = UserSession::new_default(user_id);

    let token = session.sign().unwrap();
    let expires = session.exp.unix_timestamp();

    let snowstorm_username = sanitize(User::get_id(user_id, pool).await.unwrap().username);

    (
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .embed(
                    CreateEmbed::template()
                        .title("Token Generated")
                        .description(format!(
                            "Your API key is `{token}`\nThis token will expire <t:{expires}:R>"
                        ))
                        .footer(CreateEmbedFooter::new(format!(
                            "Using linked Snowstorm account '{snowstorm_username}'"
                        ))),
                ),
        ),
        true,
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
