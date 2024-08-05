use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{types::Uuid, PgPool};

use crate::{login_controller::is_authorized, login_model::Clearance};

#[derive(Serialize)]
pub struct LogRow {
    id: i32,
    uid: String,
    key: String,
    value: String,
}

pub async fn get_logs(State(pg_pool):State<PgPool>, header_map: HeaderMap) -> Result<(StatusCode, String), (StatusCode, String)> {
    // access control
    if !is_authorized(header_map, Clearance::TopSecret) {
        return Err((
            StatusCode::UNAUTHORIZED, 
            json!({"success": false, "message": "Not authorized".to_string()}).to_string(),
        ));
    }

    let rows = sqlx::query_as!(LogRow, "SELECT id, uid, key, value FROM logs")
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| {(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )})?;
        
        Ok((
            StatusCode::OK,
            json!({"success": true, "data": rows}).to_string(),
        ))
}

#[derive(Deserialize)]
pub struct CreateLogReq {
    uid: String,
    key: String,
    value: String,
}

#[derive(Serialize)]
pub struct CreateLogRow {
    id: i32,
}

pub async fn create_log(State(pg_pool):State<PgPool>, header_map: HeaderMap, Json(user): Json<CreateLogReq>) 
  -> Result<(StatusCode, String), (StatusCode, String)> {
    // access control
    if !is_authorized(header_map, Clearance::Private) {
        return Err((
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "message": "Not authorized".to_string()}).to_string(),
        ));
    }

    let uuid = Uuid::parse_str(&user.uid).unwrap();

    let row = sqlx::query_as!(
        CreateLogRow, 
        "INSERT INTO logs (uid, key, value) VALUES ($1, $2, $3) RETURNING id",
        uuid, user.key, user.value
    ).fetch_one(&pg_pool)
    .await
    .map_err(|e| {(
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string() }).to_string(),
    )})?;

    Ok((
        StatusCode::CREATED, 
        json!({"success": true, "data": row}).to_string()
    ))
}