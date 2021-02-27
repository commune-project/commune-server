use crate::state::AppState;
use crate::apub;
use crate::errors;
use std::sync::Arc;
use serde_json::{json, Value};
use warp;

pub fn get_uri(object: Value) -> Option<String> {
    match object {
        String(s) => Some(s),
        Object(_map) => object.get("id"),
        _ => None
    }
}

pub async fn post_inbox(
    app_state: Arc<AppState>,
    domain: String,
    body: Value
) -> Result<impl warp::Reply, warp::Rejection> {
    let activity_result = serde_json::from_value::<apub::models::Activity>(body);
    match activity_result {
        Err(_e) => warp::reject::custom(errors::ActionError::InvalidForm),
        Ok(activity) => match &activity.kind {
            String::from("Create") => post_inbox_create(app_state, domain, activity).await
            String::from("Follow") => post_inbox_follow(app_state, domain, activity).await
            _ => warp::reject::custom(errors::ActionError::InvalidForm),
        }
    }
}

pub async fn post_inbox_create(
    app_state: Arc<AppState>,
    domain: String,
    activity: apub::models::Activity
) -> Result<impl warp::Reply, warp::Rejection> {
    warp::reject::custom(errors::ActionError::InternalError)
}

pub async fn post_inbox_follow(
    app_state: Arc<AppState>,
    domain: String,
    activity: apub::models::Activity
) -> Result<impl warp::Reply, warp::Rejection> {
    let object_id = &activity.object.as_str().ok_or(errors::ActionError::InvalidForm)?;

    let result = tokio::task::spawn_blocking(move || {
        actions::actor::follow_actor_by_uri(&conn, activity.actor.as_str(), object_id)
    })
    .await
    .or(Err(errors::ActionError::InternalError))??;

    match result {
        Ok(_f: Follow) => warp::reply::reply(),
        Err(error: errors::ActionError) => warp::reject::custom(error)
    }
}
