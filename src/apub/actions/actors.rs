use crate::errors::{ActionError, ActionResult};
use crate::apub::models::Actor as ActorS;
use crate::apub::webfinger::query_webfinger;
use crate::db::models::NewActor;
use crate::db::models::Actor as ActorM;
use crate::db::actions::actor::{insert_new_actor, get_actor_by_uri};
use crate::state::DbPool;
use std::convert::TryFrom;
use tokio;
use super::get_client;
use log;

/// Fetch Actor information from remote server, and store it into ActorS.
async fn fetch_actor(uri: &str) -> ActionResult<ActorS> {
    get_client()?.get(uri)
        .header("Accept", "application/activity+json")
        .send()
        .await
        .map_err(|_e| ActionError::FetchError)?
        .json::<ActorS>()
        .await
        .map_err(|_e| ActionError::FetchError)
}

/// Fetch Actor information from remote server, and store it into ActorM, then insert into database.
pub async fn fetch_actor_by_uri(db: &DbPool, uri: &str) -> ActionResult<ActorM> {
    let conn = db.get().map_err(|_e| ActionError::InternalError)?;

    let actor = fetch_actor(uri).await?;

    let webfinger_result = query_webfinger(String::from(uri)).await;

    let new_actor = match webfinger_result {
        Ok(actor_webfinger_info) => NewActor::try_from((&actor, &actor_webfinger_info)),
        Err(_e) => NewActor::try_from(&actor)
    }.map_err(|_e| ActionError::InvalidForm)?;

    
    tokio::task::spawn_blocking(move || {
        insert_new_actor(&conn, new_actor)
    })
    .await
    .map_err(|_e| ActionError::InternalError)?
    .map_err(|_e| ActionError::InsertError)
}

/// Get Actor from database, or fetch Actor information from remote server.
pub async fn get_or_fetch_actor_by_uri(db: &DbPool, uri: &str) -> ActionResult<ActorM> {
    let conn = db.get().map_err(|_e| ActionError::InternalError)?;

    let uri = String::from(uri);
    let uri2 = uri.clone();

    let result = tokio::task::spawn_blocking(move || {
        get_actor_by_uri(&conn, uri.as_str())
    })
    .await
    .map_err(|_e| ActionError::InternalError)?;

    match result {
        Ok(actor) => Ok(actor),
        Err(ActionError::NotFound) => fetch_actor_by_uri(db, uri2.as_str()).await,
        Err(err) => Err(err),
    }
}
