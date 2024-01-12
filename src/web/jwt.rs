use axum::{headers, TypedHeader};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    user_id: i64,
    exp: OffsetDateTime,
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
            &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
        )
    }

    pub fn read(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.set_required_spec_claims::<String>(&[]);
        Ok(jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
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
}
