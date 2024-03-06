use crate::server::PingResult;

use super::DbPush;
use serde::Serialize;
use sqlx::{PgPool, Row};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JoinResult {
    pub id: Option<i64>,
    // server foreign key
    pub server_id: i64,
    // join results
    pub online_mode: Option<bool>,
    pub whitelist: Option<bool>,
    pub bunger: Option<bool>,
    pub kick_message: Option<String>,
    pub honeypot: i8,
    // timestamps
    pub first_joined: i64,
    pub last_joined: i64,
}

impl JoinResult {
    #[inline]
    pub fn is_honeypot(&self) -> bool {
        self.honeypot != 0
    }

    pub async fn from_server_id(server_id: i64, pool: &PgPool) -> Option<Self> {
        const QUERY_STRING: &str = "
        SELECT * FROM server_joins WHERE server_id = $1::BIGINT;
        ";
        sqlx::query_as(QUERY_STRING)
            .bind(server_id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn from_id(id: i64, pool: &PgPool) -> Option<Self> {
        const QUERY_STRING: &str = "
        SELECT * FROM server_joins WHERE id = $1::BIGINT;
        ";
        sqlx::query_as(QUERY_STRING)
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn get_server(&self, pool: &PgPool) -> PingResult {
        PingResult::from_id(self.server_id, pool).await.unwrap()
    }
}

impl DbPush for JoinResult {
    async fn push(&mut self, pool: &sqlx::PgPool) -> Result<(), eyre::Report> {
        let query = "INSERT INTO servers (
                    server_id,
                    online_mode,
                    whitelist,
                    bunger,
                    kick_message,
                    honeypot
                ) VALUES (
                    $2::BIGINT,
                    $3::BOOLEAN,
                    $4::BOOLEAN,
                    $5::BOOLEAN,
                    $6::TEXT,
                    $7::BIT(8)
                ) ON CONFLICT (server_id) DO UPDATE SET
                    online_mode = excluded.online_mode,
                    whitelist = excluded.whitelist,
                    bunger = excluded.bunger,
                    kick_message = excluded.kick_message,
                    honeypot = excluded.honeypot,
                    last_joined = EXTRACT(epoch from now())
                RETURNING id";
        let new_id: i64 = sqlx::query(query)
            .bind(self.id)
            .bind(self.server_id)
            .bind(self.online_mode)
            .bind(self.whitelist)
            .bind(self.bunger)
            .bind(self.kick_message.as_ref())
            .bind(self.honeypot)
            .fetch_one(pool)
            .await?
            .get("id");
        self.id = Some(new_id);
        Ok(())
    }
}
