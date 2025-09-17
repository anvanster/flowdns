use actix_web::{web, HttpResponse};
use crate::api::models::{LoginRequest, RefreshTokenRequest};
use crate::api::auth::{Claims, TokenResponse, create_token, hash_password, verify_password};
use crate::api::server::ApiState;
use uuid::Uuid;
use chrono::Duration;
use tracing::{info, warn};

pub async fn login(
    state: web::Data<ApiState>,
    req: web::Json<LoginRequest>,
) -> actix_web::Result<HttpResponse> {
    // TODO: Implement proper user authentication from database
    // For now, we'll use a hardcoded example

    if req.username == "admin" && req.password == "admin123" {
        // Create access token (expires in 1 hour)
        let access_claims = Claims::new(
            Uuid::new_v4(),
            "admin".to_string(),
            Duration::hours(1),
        );

        // Create refresh token (expires in 7 days)
        let refresh_claims = Claims::new(
            Uuid::new_v4(),
            "admin".to_string(),
            Duration::days(7),
        );

        let secret = "your-secret-key"; // TODO: Get from settings

        let access_token = create_token(&access_claims, secret)
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to create token: {}", e)))?;

        let refresh_token = create_token(&refresh_claims, secret)
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to create refresh token: {}", e)))?;

        info!("User {} logged in successfully", req.username);

        Ok(HttpResponse::Ok().json(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(refresh_token),
        }))
    } else {
        warn!("Failed login attempt for user: {}", req.username);
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "invalid_credentials",
            "message": "Invalid username or password"
        })))
    }
}

pub async fn refresh(
    state: web::Data<ApiState>,
    req: web::Json<RefreshTokenRequest>,
) -> actix_web::Result<HttpResponse> {
    let secret = "your-secret-key"; // TODO: Get from settings

    // Validate refresh token
    match crate::api::auth::validate_token(&req.refresh_token, secret) {
        Ok(claims) => {
            // Create new access token
            let new_claims = Claims::new(
                Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::new_v4()),
                claims.role,
                Duration::hours(1),
            );

            let access_token = create_token(&new_claims, secret)
                .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to create token: {}", e)))?;

            info!("Token refreshed for user: {}", claims.sub);

            Ok(HttpResponse::Ok().json(TokenResponse {
                access_token,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                refresh_token: None, // Don't issue new refresh token
            }))
        }
        Err(_) => {
            warn!("Invalid refresh token attempted");
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_token",
                "message": "Invalid or expired refresh token"
            })))
        }
    }
}