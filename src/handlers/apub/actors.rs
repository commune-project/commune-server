use crate::db::actions;
use crate::state::AppState;
use crate::errors;
use crate::apub;
use crate::apub::webfinger;

use tokio;
use warp;
use warp::Reply;
use std::sync::Arc;
use serde::Deserialize;
use serde_json::Value;

pub async fn get_user(
    app_state: Arc<AppState>,
    domain: String,
    username: String
) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = app_state
        .db
        .get()
        .expect("couldn't get db connection from pool");

    let result = tokio::task::spawn_blocking(move || {
        actions::actor::get_actor_by_username_domain(&conn, username.as_str(), domain.as_str())
    })
    .await
    .or(Err(errors::ActionError::InternalError))
    .map(|result| {
        result.map_err(|e| {
            match e {
                errors::ActionError::NotFound => e,
                _ => errors::ActionError::InternalError,
            }
        }).map(|actor| {
            warp::reply::json(&apub::models::Actor::from(&actor))
        })
    });

    match result {
        Ok(Ok(body)) => Ok(body.into_response()),
        Ok(Err(err)) => Ok(err.into_response()),
        Err(err) => Ok(err.into_response())
    }
}


pub async fn get_user_outbox(
    app_state: Arc<AppState>,
    domain: String,
    username: String
) -> Result<impl warp::Reply, warp::Rejection> {
    let outbox = apub::models::Outbox {
        context: apub::serializers::get_context(),
        kind: String::from("OrderedCollection"),
        id: format!("https://{}/users/{}/inbox", domain, username),
        total_items: 0,
        ordered_items: vec![],
    };
    Ok(warp::reply::json(&outbox))
}