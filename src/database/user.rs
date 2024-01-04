use sqlx::{PgPool, Row};

use super::DbPush;

#[derive(sqlx::FromRow, Debug, Default)]
pub struct User {
    pub id: Option<i32>,
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

    pub async fn get_id(id: i32, pool: &PgPool) -> Option<Self> {
        match sqlx::query_as("SELECT * FROM users WHERE id = $1::INT")
            .bind(id)
            .fetch_optional(pool)
            .await
        {
            Ok(user) => user,
            Err(_) => None,
        }
    }

    pub async fn get_username(username: &str, pool: &PgPool) -> Option<Self> {
        match sqlx::query_as("SELECT * FROM users WHERE username ILIKE $1::TEXT")
            .bind(username)
            .fetch_optional(pool)
            .await
        {
            Ok(user) => user,
            Err(_) => None,
        }
    }
}

impl DbPush for User {
    async fn push(&mut self, pool: &PgPool) -> Result<(), eyre::Report> {
        let username = self.username.to_owned();
        let password = self.password.to_owned();

        let query = match self.id {
            Some(_) => {
                "UPDATE users SET username = $2::TEXT, password = $3::TEXT WHERE id = $1::INT"
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
            let new_id: i32 = new_id.get("id");
            self.id = Some(new_id);
        }

        Ok(())
    }
}
