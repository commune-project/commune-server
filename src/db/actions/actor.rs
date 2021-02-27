use crate::db::models::Actor;
use crate::db::models::Follow;
use crate::db::schema;
use crate::errors::{ActionError, ActionResult};
use diesel::prelude::*;
use diesel::PgConnection;

use chrono::Utc;

pub fn get_actor_by_username_domain(
    db: &PgConnection,
    username_in: &str,
    domain_in: &str,
) -> ActionResult<Actor> {
    use crate::db::schema::actors::dsl::*;
    actors
        .filter(username.eq(String::from(username_in)))
        .filter(domain.eq(String::from(domain_in)))
        .first(db)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => ActionError::NotFound,
            _ => ActionError::InternalError,
        })
}

pub fn get_actor_by_uri(db: &PgConnection, uri_in: &str) -> ActionResult<Actor> {
    use crate::db::schema::actors::dsl::*;
    actors
        .filter(uri.eq(String::from(uri_in)))
        .first(db)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => ActionError::NotFound,
            _ => ActionError::InternalError,
        })
}

pub fn follow_actor_by_uri(
    db: &PgConnection,
    follower_uri: &str,
    following_uri: &str,
) -> ActionResult<Follow> {
    let follower_actor = get_actor_by_uri(db, follower_uri)?;
    let following_actor = get_actor_by_uri(db, following_uri)?;
    let now = Utc::now().naive_utc();
    let new_follow = Follow {
        follower_id: follower_actor.id,
        following_id: following_actor.id,
        created_at: now,
        updated_at: Some(now),
        role: if following_actor.is_locked {
            String::from("pending")
        } else {
            String::from("follower")
        },
    };

    diesel::insert_into(schema::follows::table)
        .values(&new_follow)
        .get_result::<Follow>(db)
        .map_err(|_e| ActionError::InsertError)
}
