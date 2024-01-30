use super::DbPush;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct ForgejoUserInfo {
    #[serde(skip)]
    pub id: Option<i64>,
    #[serde(skip)]
    pub user_id: Option<i64>,
    #[serde(rename = "id")]
    pub forgejo_id: i64,
    #[serde(skip)]
    pub link_code: Option<String>,
    #[serde(rename = "login")]
    pub username: String,
    #[serde(rename = "full_name")]
    pub global_name: Option<String>,
    pub active: bool,
    pub is_admin: bool,
    pub prohibit_login: bool,
    pub restricted: bool,
}

impl ForgejoUserInfo {
    pub async fn get_id(id: i64, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM forgejo_users WHERE id = $1::BIGINT")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn get_forgejo_id(forgejo_id: i64, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM forgejo_users WHERE forgejo_id = $1::BIGINT")
            .bind(forgejo_id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn get_link_code(link_code: &str, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM forgejo_users WHERE link_code = $1::TEXT")
            .bind(link_code)
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

impl DbPush for ForgejoUserInfo {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        let query = match self.id {
            Some(_) => {
                "UPDATE forgejo_users SET
                    user_id = $2::BIGINT,
                    forgejo_id = $3::BIGINT,
                    link_code = $4::TEXT,
                    username = $5::TEXT,
                    global_name = $6::TEXT,
                    active = $7::BOOLEAN,
                    is_admin = $8::BOOLEAN,
                    prohibit_login = $9::BOOLEAN,
                    restricted = $10::BOOLEAN
                WHERE
                    id = $1::BIGINT"
            }
            None => {
                "INSERT INTO forgejo_users (
                    user_id,
                    forgejo_id,
                    link_code,
                    username,
                    global_name,
                    active,
                    is_admin,
                    prohibit_login,
                    restricted
                ) VALUES (
                    $2::BIGINT,
                    $3::BIGINT,
                    $4::TEXT,
                    $5::TEXT,
                    $6::TEXT,
                    $7::BOOLEAN,
                    $8::BOOLEAN,
                    $9::BOOLEAN,
                    $10::BOOLEAN
                )
                ON CONFLICT (forgejo_id) DO UPDATE SET
                    user_id = excluded.user_id,
                    link_code = excluded.link_code,
                    username = excluded.username,
                    global_name = excluded.global_name,
                    active = excluded.active,
                    is_admin = excluded.is_admin,
                    prohibit_login = excluded.prohibit_login,
                    restricted = excluded.restricted"
            }
        };
        sqlx::query(query)
            .bind(self.id)
            .bind(self.user_id)
            .bind(self.forgejo_id)
            .bind(&self.link_code)
            .bind(&self.username)
            .bind(&self.global_name)
            .bind(self.active)
            .bind(self.is_admin)
            .bind(self.prohibit_login)
            .bind(self.restricted)
            .execute(pool)
            .await?;

        if self.id.is_none() {
            let new_id = sqlx::query("SELECT id FROM forgejo_users WHERE forgejo_id = $1::BIGINT")
                .bind(self.forgejo_id)
                .fetch_one(pool)
                .await?;
            let new_id: i64 = new_id.get("id");
            self.id = Some(new_id);
        }

        Ok(())
    }
}
