use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub error: &'static str,
}

