use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq, PartialOrd)]
pub enum Clearance {
    None = 0, // for endpoints without identification / authorization
    Public = 10,
    Private = 100,
    Secret = 1000,
    TopSecret = 10000,
    Black = 100000,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub uid: i32,
    pub sub: String,
    pub prv: Clearance,
    pub exp: usize,
}