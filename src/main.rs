#[macro_use]
extern crate diesel;

use state::AppState;

mod apub;
mod db;
mod errors;
mod hancock;
mod handlers;
mod state;

use std::sync::Arc;
use warp;
use warp::Filter;
use warp::filters::BoxedFilter;

use pretty_env_logger;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let app_state = Arc::new(AppState::new());
    let app_state = warp::any().map(move || Arc::clone(&app_state));
    let with_app_state_and_host = warp::any().and(app_state.clone()).and(
        app_state
            .clone()
            .and(warp::filters::host::optional())
            .and_then(handlers::apub::set_domain),
    );
    let get_user = with_app_state_and_host
        .clone()
        .and(warp::get())
        .and(warp::path!("users" / String))
        .and_then(handlers::apub::actors::get_user);
    let get_communities = with_app_state_and_host
        .clone()
        .and(warp::get())
        .and(warp::path!("communities" / String))
        .and_then(handlers::apub::actors::get_user);

    let get_user_outbox = with_app_state_and_host
        .clone()
        .and(warp::get())
        .and(warp::path!("users" / String / "outbox"))
        .and_then(handlers::apub::actors::get_user_outbox);
    let get_communities_outbox = with_app_state_and_host
        .clone()
        .and(warp::get())
        .and(warp::path!("communities" / String / "outbox"))
        .and_then(handlers::apub::actors::get_user_outbox);

    // Actor followers
    let get_user_followers = with_app_state_and_host
        .clone()
        .and(warp::get())
        .and(warp::path!("users" / String / "followers"))
        .and(warp::query())
        .and_then(handlers::apub::actors::get_user_followers);
    let get_communities_followers = with_app_state_and_host
        .clone()
        .and(warp::get())
        .and(warp::path!("communities" / String / "followers"))
        .and(warp::query())
        .and_then(handlers::apub::actors::get_user_followers);


    let get_webfinger = with_app_state_and_host
        .clone()
        .and(warp::path!(".well-known" / "webfinger"))
        .and(warp::filters::query::query())
        .and_then(handlers::webfinger::get_webfinger)
        .map(handlers::webfinger::map_content_type_webfinger);

    let ap_routes = get_user
        .or(get_communities)
        .or(get_user_outbox)
        .or(get_user_followers)
        .or(get_communities_outbox)
        .or(get_communities_followers)
        .map(handlers::apub::map_content_type_ap);

    let post_inbox = with_app_state_and_host
        .clone()
        .and(warp::post())
        .and(warp::path("inbox").or(warp::path!("users" / String / "inbox").map(|_s| ()).untuple_one()).unify())
        .and(filter_auth_http_signatures(with_app_state_and_host.clone().boxed()))
        .and(handlers::apub::activity_json())
        .and_then(handlers::apub::inbox::post_inbox);

    let ap_routes = ap_routes.or(post_inbox);

    warp::serve(ap_routes.or(get_webfinger))
        .run(([0, 0, 0, 0], 8000))
        .await;
}

fn filter_auth_http_signatures(
    with_app_state_and_host: BoxedFilter<(Arc<AppState>, String)>
) -> BoxedFilter<(db::models::Actor,)> {
    with_app_state_and_host
        .and(warp::method())
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and_then(handlers::apub::auth::authenticate_http_signatures)
        .boxed()
}

fn filter_must_authenticate(
    auth_http_signatures: BoxedFilter<(db::models::Actor,)>,
) -> BoxedFilter<()>{
    auth_http_signatures.and(handlers::apub::activity_json()).and_then(|actor, body| async move {
        handlers::apub::auth::must_authenticate(actor, body)
    })
    .untuple_one()
    .boxed()
}
