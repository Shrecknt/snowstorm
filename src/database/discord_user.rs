use serde::{Deserialize, Serialize};
use sqlx::Row;

use super::DbPush;

#[derive(Deserialize, Serialize, Debug)]
pub struct DiscordUserInfo {
    pub id: Option<i32>,
    pub user_id: Option<i32>,
    pub link_code: Option<String>,
    #[serde(rename = "id")]
    pub discord_id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub public_flags: Option<i32>,
    pub premium_type: Option<i32>,
    pub flags: Option<i32>,
    pub banner: Option<String>,
    pub accent_color: Option<i32>,
    pub global_name: Option<String>,
    pub avatar_decoration: Option<String>,
    pub banner_color: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub locale: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub verified: Option<bool>,
    pub email: Option<String>,
}

impl DbPush for DiscordUserInfo {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        let query = match self.id {
            Some(_) => {
                "UPDATE discord_users SET
                    user_id = $2::INT,
                    discord_id = $3::TEXT,
                    username = $4::TEXT,
                    discriminator = $5::TEXT,
                    global_name = $6::TEXT,
                    link_code = $7::TEXT,
                WHERE
                    id = $1::SERIAL"
            }
            None => {
                "INSERT INTO discord_users (
                    user_id,
                    discord_id,
                    username,
                    discriminator,
                    global_name,
                    link_code
                ) VALUES (
                    $2::INT,
                    $3::TEXT,
                    $4::TEXT,
                    $5::TEXT,
                    $6::TEXT,
                    $7::TEXT
                )"
            }
        };
        sqlx::query(query)
            .bind(self.id)
            .bind(self.user_id)
            .bind(&self.discord_id)
            .bind(&self.username)
            .bind(&self.discriminator)
            .bind(&self.global_name)
            .bind(&self.link_code)
            .execute(pool)
            .await?;

        if self.id.is_none() {
            let new_id = sqlx::query("SELECT id FROM discord_users WHERE discord_id = $1::TEXT")
                .bind(&self.discord_id)
                .fetch_one(pool)
                .await?;
            let new_id: i32 = new_id.get("id");
            self.id = Some(new_id);
        }

        Ok(())
    }
}
