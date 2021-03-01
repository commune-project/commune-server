use crate::db::schema::actors;
use chrono;
use chrono::prelude::Utc;
use serde::{Deserialize, Serialize};
use crate::apub::username::username_to_idna;

use validator::Validate;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[table_name = "actors"]
pub struct Actor {
    #[serde(skip)]
    pub id: i64,
    pub uri: String,
    pub url: Option<String>,
    pub kind: String,

    pub username: String,
    pub domain: String,
    pub name: String,
    pub summary: String,
    pub avatar_url: String,

    pub inbox_uri: String,
    pub outbox_uri: String,
    pub followers_uri: Option<String>,
    pub following_uri: Option<String>,
    pub public_key_pem: String,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,

    pub lang: String,
    pub is_locked: bool,
    pub is_suspended: bool,
    pub is_silenced: bool,
}

#[derive(Clone, Insertable, PartialEq, Debug, Deserialize, Default, Validate)]
#[table_name = "actors"]
#[serde(rename_all = "camelCase")]
pub struct NewActor {
    #[validate(url)]
    pub uri: String,
    #[validate(url)]
    pub url: Option<String>,
    pub kind: String,

    #[validate(non_control_character)]
    pub username: String,
    pub domain: String,
    #[validate(non_control_character)]
    pub name: Option<String>,
    #[validate(non_control_character)]
    pub summary: Option<String>,

    #[validate(url)]
    pub avatar_url: Option<String>,

    #[validate(url)]
    pub inbox_uri: String,
    #[validate(url)]
    pub outbox_uri: String,
    #[validate(url)]
    pub followers_uri: Option<String>,
    #[validate(url)]
    pub following_uri: Option<String>,
    pub public_key_pem: String,

    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,

    pub lang: String,
    pub is_locked: bool,
    pub is_suspended: bool,
    pub is_silenced: bool,
}

pub enum ActorType {
    Person,
    Service,
    Application,
    Group,
}

impl From<&Actor> for ActorType {
    fn from(actor: &Actor) -> ActorType {
        match actor.kind.as_str() {
            "Service" => ActorType::Service,
            "Application" => ActorType::Application,
            "Group" => ActorType::Group,
            _ => ActorType::Person,
        }
    }
}

impl From<&ActorType> for String {
    fn from(actor_type: &ActorType) -> String {
        match actor_type {
            ActorType::Person => String::from("Person"),
            ActorType::Service => String::from("Service"),
            ActorType::Application => String::from("Application"),
            ActorType::Group => String::from("Group"),
        }
    }
}

pub struct NewLocalActorBuilder<'a> {
    pub username: &'a str,
    pub domain: &'a str,
    pub lang: &'a str,
    pub actor_type: ActorType,
    pub public_key_pem: &'a str,
}

impl NewLocalActorBuilder<'_> {
    pub fn build(&self) -> NewActor {
        let now = Utc::now().naive_utc();
        NewActor {
            kind: String::from(&self.actor_type),
            username: String::from(self.username),
            domain: String::from(self.domain),
            uri: self.uri(),
            url: self.url(),
            inbox_uri: self.inbox_uri(),
            outbox_uri: self.outbox_uri(),
            followers_uri: self.followers_uri(),
            following_uri: self.following_uri(),
            created_at: Some(now),
            updated_at: Some(now),
            public_key_pem: String::from(self.public_key_pem),

            ..Default::default()
        }
    }

    fn uri(&self) -> String {
        let slug = match &self.actor_type {
            ActorType::Group => "communities",
            _ => "users",
        };
        format!("https://{}/{}/{}", self.domain, slug, username_to_idna(self.username))
    }

    fn inbox_uri(&self) -> String {
        format!("{}/inbox", self.uri())
    }

    fn outbox_uri(&self) -> String {
        format!("{}/outbox", self.uri())
    }

    fn following_uri(&self) -> Option<String> {
        match &self.actor_type {
            ActorType::Group => None,
            _ => Some(format!("{}/following", self.uri())),
        }
    }

    fn followers_uri(&self) -> Option<String> {
        Some(format!("{}/followers", self.uri()))
    }

    fn url(&self) -> Option<String> {
        Some(format!("https://{}/@{}", self.domain, self.username))
    }
}