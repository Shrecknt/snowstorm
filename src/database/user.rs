use super::DbPush;
use sqlx::{PgPool, Row};

#[derive(sqlx::FromRow, Debug, Default)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
}

impl User {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            id: None,
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub async fn get_id(id: i64, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM users WHERE id = $1::BIGINT")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }

    pub async fn get_username(username: &str, pool: &PgPool) -> Option<Self> {
        sqlx::query_as("SELECT * FROM users WHERE username ILIKE $1::TEXT")
            .bind(username)
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

impl DbPush for User {
    async fn push(&mut self, pool: &PgPool) -> Result<(), eyre::Report> {
        let username = self.username.to_owned();
        let password = self.password.to_owned();

        let query = match self.id {
            Some(_) => {
                "UPDATE users SET username = $2::TEXT, password = $3::TEXT WHERE id = $1::BIGINT"
            }
            None => "INSERT INTO users (username, password) VALUES ($2::TEXT, $3::TEXT)",
        };
        sqlx::query(query)
            .bind(self.id)
            .bind(&username)
            .bind(&password)
            .execute(pool)
            .await?;

        if self.id.is_none() {
            let new_id = sqlx::query("SELECT id FROM users WHERE username = $1::TEXT")
                .bind(&username)
                .fetch_one(pool)
                .await?;
            let new_id: i64 = new_id.get("id");
            self.id = Some(new_id);
        }

        Ok(())
    }
}
