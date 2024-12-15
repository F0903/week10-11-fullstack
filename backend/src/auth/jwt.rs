use super::AUTH_COOKIE_NAME;
use crate::{
    api::v1::{ApiError, ApiResponse},
    utils::generic_result::GenericResult,
};
use chrono::Utc;
use dotenv_codegen::dotenv;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    serde::{Deserialize, Serialize},
};

const JWT_ALGORITHM: Algorithm = Algorithm::HS512;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: i32,
    exp: usize,
}

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims,
}

// Implement request guard for JWT
#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = ApiResponse<ApiError>;

    async fn from_request(req: &'r request::Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookies = req.cookies();
        let auth_cookie = match cookies.get(AUTH_COOKIE_NAME) {
            Some(x) => x,
            None => return Outcome::Error((Status::Unauthorized, ApiResponse::unauthorized())),
        };

        let key = auth_cookie.value();

        let claims = match decode_jwt_token(key) {
            Ok(x) => x,
            Err(_) => return Outcome::Error((Status::Unauthorized, ApiResponse::unauthorized())),
        };

        Outcome::Success(JWT { claims })
    }
}

pub fn encode_jwt_token(user_id: i32) -> GenericResult<String> {
    let secret: &str = dotenv!("JWT_SECRET");

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(6))
        .expect("Invalid timestamp")
        .timestamp();

    let claims = Claims {
        user_id,
        exp: expiration as usize,
    };

    let header = Header::new(JWT_ALGORITHM);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn decode_jwt_token(token: &str) -> Result<Claims, jsonwebtoken::errors::ErrorKind> {
    let secret: &str = dotenv!("JWT_SECRET");

    let token = jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(JWT_ALGORITHM),
    )
    .map_err(|e| e.kind().to_owned())?;

    Ok(token.claims)
}
