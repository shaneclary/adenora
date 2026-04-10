use crate::state::AppState;
use adenora_users::auth::validate_token;
use axum::{
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
};
use uuid::Uuid;

/// Extracts authenticated user from JWT Bearer token.
pub struct AuthUser {
    pub user_id: Uuid,
    pub is_bot: bool,
    pub is_admin: bool,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing authorization header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "invalid authorization format"))?;

        let claims = validate_token(token, &state.config.auth.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid user ID in token"))?;

        Ok(AuthUser {
            user_id,
            is_bot: claims.is_bot,
            is_admin: claims.is_admin,
        })
    }
}
