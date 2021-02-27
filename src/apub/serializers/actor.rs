use crate::db::models::Actor;
use super::ImageSerializer;
use super::get_context;
use crate::apub;

use serde_json::{
    json,
    value::Value
};

pub struct ActorSerializer {
    actor: Actor,
}

impl ActorSerializer {
    pub fn new(actor: Actor) -> Self {
        ActorSerializer {
            actor: actor,
        }
    }

    pub fn serialize(&self) -> Value {
        json!({
            "@context": get_context(),
            "id": self.actor.uri,
            "type": self.actor.kind,
            "url": self.actor.url,
            "preferredUsername": self.actor.username,
            "name": self.actor.name,
            "summary": self.actor.summary,
            "manuallyApprovesFollowers": self.actor.is_locked,
            "publicKey": {
                "id": format!("{}#main-key", self.actor.uri),
                "owner": self.actor.uri,
                "publicKeyPem": self.actor.public_key_pem
            },
            "endpoints": {
                "sharedInbox": self.get_shared_inbox()
            },
            "icon": ImageSerializer::new(self.actor.avatar_url.as_str()).serialize()
        })
    }

    fn get_shared_inbox(&self) -> String {
        format!("https://{}/inbox", self.actor.domain)
    }
}