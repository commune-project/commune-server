use crate::db;
use crate::db::actions;
use crate::state::AppState;
use crate::errors;
use crate::errors::ActionError;
use crate::apub;
use crate::apub::models::PagedCollection;

use tokio;
use warp;
use warp::Reply;
use std::sync::Arc;
use serde_json::json;

pub async fn get_user(
    app_state: Arc<AppState>,
    domain: String,
    username: String
) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = app_state.db.get().map_err(|_e| warp::reject())?;

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
    _app_state: Arc<AppState>,
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

pub async fn get_user_followers(
    app_state: Arc<AppState>,
    domain: String,
    username: String,
    paged_collection: apub::models::PagedCollection
) -> Result<impl warp::Reply, warp::Rejection> {
    if paged_collection.is_paged() {
        get_user_followers_paged(app_state, domain, username, paged_collection).await
    } else {
        get_user_followers_not_paged(app_state, domain, username, paged_collection).await
    }
}

async fn get_user_followers_not_paged(
    app_state: Arc<AppState>,
    domain: String,
    username: String,
    _paged_collection: apub::models::PagedCollection
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let conn = app_state.db.get().map_err(|_e| warp::reject::custom(ActionError::InternalError))?;
    
    let username_move = username.clone();
    let domain_move = domain.clone();

    let total_items = tokio::task::spawn_blocking(move || {
        let actor = actions::actor::get_actor_by_username_domain(&conn, username_move.as_str(), domain_move.as_str())?;
        let total_items = actions::follow::actor_count_followers(&conn, &actor)?;
        Ok(total_items)
    })
    .await
    .unwrap_or(Err(ActionError::InternalError))
    .map_err(|err| warp::reject::custom(err))?;

    Ok(Box::new(warp::reply::json(&json!({
        "@context": apub::serializers::get_context(),
        "type": "OrderedCollection",
        "id": format!("https://{}/users/{}/followers", domain, username),
        "totalItems": total_items,
        "first": format!("https://{}/users/{}/followers?page=1", domain, username)
    }))))
}

async fn get_user_followers_paged(
    app_state: Arc<AppState>,
    domain: String,
    username: String,
    paged_collection: apub::models::PagedCollection
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let conn = app_state.db.get().map_err(|_e| warp::reject::custom(ActionError::InternalError))?;

    let username_move = username.clone();
    let domain_move = domain.clone();
    let page_number = paged_collection.page_number();

    let (total_items, actor_id_vec) = tokio::task::spawn_blocking(move || {
        let actor = actions::actor::get_actor_by_username_domain(&conn, username_move.as_str(), domain_move.as_str())?;
        let followers = actions::follow::actor_get_followers(&conn, &actor, page_number)?;
        let total_items = actions::follow::actor_count_followers(&conn, &actor)?;
        Ok((total_items, followers))
    })
    .await
    .unwrap_or(Err(ActionError::InternalError))
    .map_err(|err| warp::reject::custom(err))
    .map(|(total_items, actor_vec)| {
        let actor_id_vec = actor_vec.iter().map(|actor| {
            actor.uri.clone()
        }).collect::<Vec<String>>();

        (total_items, actor_id_vec)
    })?;

    let next = if paged_collection.has_next(total_items, db::actions::follow::PAGE_SIZE) {
        Some(format!("https://{}/users/{}/followers?page={}", domain, username, paged_collection.next_page_number()))
    } else {
        None
    };
    let prev = if paged_collection.has_prev() {
        Some(format!("https://{}/users/{}/followers?page={}", domain, username, paged_collection.prev_page_number()))
    } else {
        None
    };

    Ok(Box::new(warp::reply::json(&json!({
        "@context": apub::serializers::get_context(),
        "type": "OrderedCollectionPage",
        "id": format!("https://{}/users/{}/followers?page={}", domain, username, paged_collection.page_number()),
        "next": next,
        "prev": prev,
        "totalItems": total_items,
        "orderedItems": actor_id_vec,
    }))))
}