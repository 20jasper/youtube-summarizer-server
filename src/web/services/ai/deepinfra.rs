use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: String,
    pub content: String,
}
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Choice {
    pub message: Message,
}
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Response {
    pub choices: Vec<Choice>,
}
