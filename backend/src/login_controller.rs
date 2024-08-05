use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::json;
use sqlx::PgPool;

use crate::login_model::{Claims, Clearance, LoginInfo, LoginResponse};

pub async fn login_handler(State(pg_pool):State<PgPool>, Json(login_info) : Json<LoginInfo>) -> Result<Json<LoginResponse>, StatusCode> {
    // get the login information from the post request
    let username = &login_info.username;
    let password = &login_info.password;

    // build the database query
    let query = sqlx::query!("SELECT id, username, clearance FROM users WHERE username = $1 AND password = $2", username, password);
     
    let result = query.fetch_optional(&pg_pool).await.map_err(|e| {(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"success": false, "message": e.to_string()}).to_string(),
        )
    }).unwrap();

    let is_valid = result.is_some();
    if is_valid {
        let row = result.unwrap();

        let claims = Claims {
            uid: row.id,
            sub: username.clone(),
            prv: unsafe { ::std::mem::transmute(row.clearance) },
            exp: (chrono::Utc::now() + chrono::Duration::hours(8)).timestamp() as usize
        };

        let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref())) {
            Ok(tok) => tok,
            Err(e) => {
                 eprintln!("Error generating Token: {}", e.to_string());
                 return Err(StatusCode::INTERNAL_SERVER_ERROR)
            },
        };

        return Ok(Json(LoginResponse{token}));
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}


pub async fn get_info_handler(header_map: HeaderMap) -> Result<Json<String>, StatusCode> {
    // access control
    if !is_authorized(header_map, Clearance::Secret) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    return Ok(axum::Json(json!({"success" : true}).to_string()));
}

pub fn is_authorized(header_map: HeaderMap, clearance_required: Clearance) -> bool {
    // for API endpoints where no identification is required
    if clearance_required == Clearance::None {
        return true;
    }

    if let Some(auth_header) = header_map.get("Authorization") {
        if let Ok(auth_header_str) = auth_header.to_str() {
            if auth_header_str.starts_with("Bearer ") {
                let token = auth_header_str.trim_start_matches("Bearer ").to_string();

                match decode::<Claims>(&token, &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()), &Validation::default()) {
                    Ok(token_data) => {
                        let claims = token_data.claims;
                        return clearance_required <= claims.prv;
                    },
                    Err(_) => {
                        return false;
                    }
                }
            }
        }
    }

    false
}