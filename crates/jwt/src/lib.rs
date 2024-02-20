use axum::{headers, TypedHeader};
use database::user::User;
use dotenvy_macro::dotenv as var;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: i64,
    pub exp: OffsetDateTime,
}

impl UserSession {
    pub fn new_default(user_id: i64) -> Self {
        Self::new(user_id, OffsetDateTime::now_utc() + Duration::hours(3))
    }

    pub fn new(user_id: i64, exp: OffsetDateTime) -> Self {
        Self { user_id, exp }
    }

    pub fn is_valid(&self) -> bool {
        OffsetDateTime::now_utc() < self.exp
    }

    pub fn sign(&self) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(var!("JWT_SECRET").as_ref()),
        )
    }

    pub fn read(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.set_required_spec_claims::<String>(&[]);
        Ok(jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(var!("JWT_SECRET").as_ref()),
            &validation,
        )?
        .claims)
    }

    pub fn from_cookies(
        cookies: &Option<TypedHeader<headers::Cookie>>,
    ) -> Option<Result<Self, jsonwebtoken::errors::Error>> {
        if let Some(TypedHeader(cookies)) = cookies {
            let authentication_cookie = match cookies.get("__Host-Authentication") {
                Some(cookie) => cookie,
                None => return None,
            };
            Some(Self::read(authentication_cookie))
        } else {
            None
        }
    }

    pub async fn user(&self, pool: &PgPool) -> User {
        User::get_id(self.user_id, pool)
            .await
            .expect("no user found for valid user id")
    }
}
