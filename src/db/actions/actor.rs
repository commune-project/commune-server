use crate::db::models::{Actor, NewActor};
use crate::errors::{ActionError, ActionResult};
use diesel::prelude::*;
use diesel::PgConnection;
use validator::Validate;

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

pub fn insert_new_actor(db: &PgConnection, new_actor: NewActor) -> ActionResult<Actor> {
    use crate::db::schema::actors::dsl::*;
    new_actor.validate().map_err(|_e| ActionError::InvalidForm)?;
    diesel::insert_into(actors)
        .values(&new_actor)
        .get_result::<Actor>(db)
        .map_err(|_e| {
            ActionError::InsertError
        })
}