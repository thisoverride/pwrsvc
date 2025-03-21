use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub r#type: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestData {
    pub source: String,
    pub version: f64,
    pub request: Request
}

// Structure de réponse
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseData {
    pub status: String,
    pub message: String,
    pub code: i32
}