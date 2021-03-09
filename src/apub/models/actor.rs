use super::Activity;
use crate::apub::username::*;
use crate::db;
use crate::errors;
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::convert::TryFrom;
use crate::apub::serializers::get_context;
use super::empty_string_or_none;
use crate::apub::webfinger;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    // Properties according to
    // - https://www.w3.org/TR/activitypub/#actor-objects
    // - https://www.w3.org/TR/activitystreams-core/#actors
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub summary: Option<String>,
    pub following: Option<String>,
    pub followers: Option<String>,
    pub inbox: String,
    pub outbox: String,
    pub preferred_username: String,
    pub name: Option<String>,
    pub public_key: serde_json::Value,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<serde_json::Value>,
    pub endpoints: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manually_approves_followers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspended: Option<bool>,
}

impl Actor {
    pub fn get_public_key_pem(&self) -> Option<String> {
        if self.public_key["owner"] == json!(self.id) && self.public_key["id"] == json!(format!("{}#main-key", self.id)) {
            self.public_key["publicKeyPem"].as_str().map(String::from)
        } else {
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outbox {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub total_items: i64,
    pub ordered_items: Vec<Activity>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Followers {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub total_items: i64,
    pub ordered_items: Vec<String>,
}


impl From<&db::models::Actor> for Actor {
    fn from(actor_db: &db::models::Actor) -> Self {
        let url = match &actor_db.url {
            Some(s) => s.clone(),
            None => actor_db.uri.clone(),
        };
        Actor {
            context: Some(get_context()),
            kind: actor_db.kind.clone(),
            id: actor_db.uri.clone(),
            url,
            preferred_username: username_to_idna(actor_db.username.as_str()),
            name: empty_string_or_none(actor_db.name.clone()),
            summary: empty_string_or_none(actor_db.summary.clone()),
            following: actor_db.following_uri.clone(),
            followers: actor_db.followers_uri.clone(),
            inbox: actor_db.inbox_uri.clone(),
            outbox: actor_db.outbox_uri.clone(),
            public_key: json!({
                "id": format!("{}#main-key", actor_db.uri),
                "owner": actor_db.uri,
                "publicKeyPem": actor_db.public_key_pem
            }),
            endpoints: Some(json!({
                "sharedInbox": format!("https://{}/inbox", actor_db.domain)
            })),
            manually_approves_followers: Some(actor_db.is_locked.clone()),
            icon: Some(json!({
                "type": "Image",
                "url": actor_db.avatar_url
            })),
            suspended: Some(actor_db.is_suspended.clone()),
        }
    }
}

impl TryFrom<&Actor> for db::models::NewActor {
    type Error = errors::ActionError;
    fn try_from(actor_ap: &Actor) -> Result<Self, Self::Error> {
        let id_domain = String::from(
            url::Url::parse(actor_ap.id.as_str())
                .or(Err(errors::ActionError::InvalidForm))?
                .host_str()
                .ok_or(errors::ActionError::InvalidForm)?,
        );
        let avatar_url = match &actor_ap.icon {
            Some(object) => object["url"].as_str().map(String::from),
            None => None,
        };
        let is_locked = actor_ap.manually_approves_followers.unwrap_or(false);
        let is_suspended = actor_ap.suspended.unwrap_or(false);
        Ok(db::models::NewActor {
            uri: actor_ap.id.clone(),
            url: Some(actor_ap.url.clone()),
            kind: actor_ap.kind.clone(),
            username: idna_to_username(actor_ap.preferred_username.as_str()),
            domain: id_domain,
            name: actor_ap.name.clone(),
            summary: actor_ap.summary.clone(),

            avatar_url: avatar_url,
            inbox_uri: actor_ap.inbox.clone(),
            outbox_uri: actor_ap.outbox.clone(),
            followers_uri: actor_ap.followers.clone(),
            following_uri: actor_ap.following.clone(),
            created_at: None,
            updated_at: None,

            public_key_pem: actor_ap.get_public_key_pem().ok_or(errors::ActionError::InvalidForm)?.clone(),

            lang: String::from("und"),
            is_locked,
            is_suspended,
            is_silenced: false,
        })
    }
}

impl TryFrom<(&Actor, &webfinger::WebfingerInfo)> for db::models::NewActor {
    type Error = errors::ActionError;
    fn try_from(actor: (&Actor, &webfinger::WebfingerInfo)) -> Result<Self, Self::Error> {
        let new_actor = Self::try_from(actor.0)?;
        Ok(db::models::NewActor {
            domain: actor.1.acct.clone().domain,
            ..new_actor
        })
    }
}
