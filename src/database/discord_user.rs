use super::DbPush;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct DiscordUserInfo {
    #[serde(skip)]
    pub id: Option<i64>,
    #[serde(skip)]
    pub user_id: Option<i64>,
    #[serde(skip)]
    pub link_code: Option<String>,
    #[serde(rename = "id")]
    pub discord_id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    // pub avatar: Option<String>,
    // pub public_flags: Option<i32>,
    // pub premium_type: Option<i32>,
    // pub flags: Option<i32>,
    // pub banner: Option<String>,
    // pub accent_color: Option<i32>,
    // pub avatar_decoration: Option<String>,
    // pub banner_color: Option<String>,
    // pub mfa_enabled: Option<bool>,
    // pub locale: Option<String>,
    // pub bot: Option<bool>,
    // pub system: Option<bool>,
    // pub verified: Option<bool>,
    // pub email: Option<String>,
}

impl DiscordUserInfo {
    pub async fn get_id(id: i64, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM discord_users WHERE id = $1::BIGINT")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn get_discord_id(discord_id: &str, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM discord_users WHERE discord_id = $1::TEXT")
            .bind(discord_id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn get_link_code(link_code: &str, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM discord_users WHERE link_code = $1::TEXT")
            .bind(link_code)
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

impl DbPush for DiscordUserInfo {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        let query = match self.id {
            Some(_) => {
                "UPDATE discord_users SET
                    user_id = $2::BIGINT,
                    discord_id = $3::TEXT,
                    username = $4::TEXT,
                    discriminator = $5::TEXT,
                    global_name = $6::TEXT,
                    link_code = $7::TEXT
                WHERE
                    id = $1::BIGINT"
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
                    $2::BIGINT,
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
            let new_id: i64 = new_id.get("id");
            self.id = Some(new_id);
        }

        Ok(())
    }
}
