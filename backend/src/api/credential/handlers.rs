//! Handler functions for credential management API endpoints.
//!
//! These functions process requests for credential data, interact with the database
//! or relevant services, and return credential-specific information.

use crate::api::common::ApiResponse;
use crate::repositories::credential_repository::CredentialRepository;
use crate::utils::jwt::Claims;
use axum::{Json, extract::Extension, http::StatusCode};
use sqlx::SqlitePool;

/// Response structure for credential status
#[derive(Debug, serde::Serialize)]
pub struct CredentialStatus {
    pub has_credential: bool,
    pub node_id: Option<String>,
    pub node_alias: Option<String>,
}

/// Get the credential status for the authenticated user
#[axum::debug_handler]
pub async fn get_user_credential_status(
    Extension(pool): Extension<SqlitePool>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<CredentialStatus>>, (StatusCode, String)> {
    let repo = CredentialRepository::new(&pool);

    match repo.get_credential_by_user_id(&claims.sub).await {
        Ok(Some(credential)) => {
            let status = CredentialStatus {
                has_credential: true,
                node_id: Some(credential.node_id),
                node_alias: Some(credential.node_alias),
            };
            Ok(Json(ApiResponse::success(
                status,
                "Credential status retrieved successfully",
            )))
        }
        Ok(None) => {
            let status = CredentialStatus {
                has_credential: false,
                node_id: None,
                node_alias: None,
            };
            Ok(Json(ApiResponse::success(
                status,
                "No credential found for user",
            )))
        }
        Err(e) => {
            tracing::error!("Failed to get credential status: {}", e);
            let error_response = ApiResponse::<()>::error(
                "Failed to retrieve credential status".to_string(),
                "database_error",
                None,
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::to_string(&error_response).unwrap(),
            ))
        }
    }
}
