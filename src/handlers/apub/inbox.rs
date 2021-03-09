use crate::apub;
use crate::db::actions;
use crate::db::models::Actor as ActorM;
use crate::apub::models::Activity as ActivityS;
use crate::state::AppState;
use crate::errors::ActionError;
use serde_json::Value;
use std::sync::Arc;
use warp;

pub fn get_uri(object: Value) -> Option<String> {
    match &object {
        Value::String(s) => Some(s.clone()),
        Value::Object(_map) => object
            .clone()
            .get("id")
            .map(|v: &Value| v.as_str())
            .unwrap_or(None)
            .map(|s: &str| String::from(s)),
        _ => None,
    }
}

pub async fn post_inbox(
    app_state: Arc<AppState>,
    domain: String,
    actor: ActorM,
    body: Value,
) -> Result<impl warp::Reply, warp::Rejection> {
    let activity = serde_json::from_value::<ActivityS>(body)
        .map_err(|_e| warp::reject::custom(ActionError::InvalidForm))?;
    if actor.uri == activity.actor {
        match activity.kind.as_str() {
            "Create" => post_inbox_create(app_state, domain, activity).await,
            "Follow" => post_inbox_follow(app_state, domain, activity).await,
            "Undo" => post_inbox_undo(app_state, domain, activity).await,
            _ => Err(warp::reject()),
        }
    } else {
        Err(warp::reject::custom(ActionError::NotAuthenticated))
    }
}

pub async fn post_inbox_create(
    _app_state: Arc<AppState>,
    _domain: String,
    _activity: ActivityS,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    Err(warp::reject())
}

pub async fn post_inbox_follow(
    app_state: Arc<AppState>,
    _domain: String,
    activity: ActivityS,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let actor_id = activity.actor.clone();
    let object_id = activity.object.clone();
    let object_id = object_id.as_str().ok_or(warp::reject())?.to_owned();
    let actor_id = actor_id.as_str().to_owned();

    let conn = app_state.db.get().map_err(|_e| warp::reject())?;

    let _result = tokio::task::spawn_blocking(move || {
        actions::follow::follow_actor_by_uri(&conn, &actor_id, &object_id).or(Err(warp::reject()))
    })
    .await
    .unwrap_or(Err(warp::reject()))?;

    Ok(Box::new(warp::reply()))
}

async fn post_inbox_undo(
    app_state: Arc<AppState>,
    domain: String,
    activity: ActivityS,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let object_activity: ActivityS = serde_json::from_value(activity.object.clone()).or(Err(warp::reject::custom(ActionError::InvalidForm)))?;

    match object_activity.kind.as_str() {
        "Follow" => post_inbox_undo_follow(app_state, domain, activity).await,
        _ => Err(warp::reject()),
    }
}

async fn post_inbox_undo_follow(
    app_state: Arc<AppState>,
    _domain: String,
    activity: ActivityS,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let actor_id = activity.actor.clone();
    let object_activity: ActivityS = serde_json::from_value(activity.object.clone()).or(Err(warp::reject::custom(ActionError::InvalidForm)))?;
    let object_id = object_activity.object.as_str().ok_or(warp::reject())?.to_owned();

    let conn = app_state.db.get().map_err(|_e| warp::reject())?;

    tokio::task::spawn_blocking(move || {
        actions::follow::unfollow_actor_by_uri(&conn, &actor_id, &object_id).map_err(|err| warp::reject::custom(err))
    })
    .await
    .unwrap_or(Err(warp::reject()))?;

    Ok(Box::new(warp::reply()))
}