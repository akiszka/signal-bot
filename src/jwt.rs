use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    exp: usize,  // expiry time (unix timestamp)
    iat: usize,  // issued at time (unix timestamp)
    sub: String, // user id / phone number of user -- subject (whom this token refers to)
}

/// This struct represents the JWT token.
/// It can be used as a request guard.
/// The token has to be supplied through:
/// - the "Authorization" header or
/// - the "token" query parameter
struct UserToken {
    token: String,
    claims: Claims,
}

static JWT_SECRET: &[u8] = "secret".as_bytes();

impl UserToken {
    /// This issues a token for a given phone number.
    pub fn issue(phone_number: String) -> Result<Self, Box<dyn Error>> {
        let claims = Claims {
            exp: 0,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs()
                .try_into()?,
            sub: phone_number,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )?;

        Ok(UserToken { token, claims })
    }

    pub fn decode(token: String) -> Result<Self, Box<dyn Error>> {
        let mut validation = Validation::default();
        validation.validate_exp = false;

        let claims = decode::<Claims>(&token, &DecodingKey::from_secret(JWT_SECRET), &validation)?;

        Ok(UserToken {
            token,
            claims: claims.claims,
        })
    }

    /// This returns the user the token was issued for.
    pub fn get_user(&self) -> &str {
        &self.claims.sub
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserToken {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let token_header = request
            .headers()
            .get_one("Authorization")
            .map(|val| val.replace("Bearer ", ""));

        let token_url = request
            .query_value::<String>("token")
            .map(|val| val.ok())
            .unwrap_or(None);

        if token_header.is_none() && token_url.is_none() {
            return Outcome::Failure((Status::BadRequest, "No token supplied"));
        } else if token_header.is_some() && token_url.is_some() {
            return Outcome::Failure((Status::BadRequest, "Too many tokens supplied"));
        }

        let token = token_header.unwrap_or(token_url.unwrap_or("".into()));

        match UserToken::decode(token) {
            Ok(token) => Outcome::Success(token),
            Err(_) => Outcome::Failure((Status::Unauthorized, "Invalid token")),
        }
    }
}
