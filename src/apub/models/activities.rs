use serde::{Deserialize, Serialize};
use serde_json::{self, Value};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    // Properties according to
    // - https://www.w3.org/TR/activitystreams-core/#activities
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub actor: String,
    pub object: serde_json::Value,
    pub published: String,
    pub to: Value,
    pub cc: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub published: String,
    pub attributed_to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub to: Value,
    pub cc: Value,
    pub tag: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitive: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub kind: String,
    pub href: String,
    pub name: String,
}
