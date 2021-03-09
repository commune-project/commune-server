use crate::apub;
use crate::db::actions;
use crate::db::models::Actor;
use crate::errors;
use crate::hancock;
use crate::state::AppState;
use crate::apub::models::Activity;
use log;
use openssl;
use std::sync::Arc;
use url;
use warp;
use warp::filters::path::FullPath;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::http::Method;

pub fn do_verify(
    key: &openssl::pkey::PKey<openssl::pkey::Public>,
    alg: openssl::hash::MessageDigest,
    src: &[u8],
    sig: &[u8],
) -> Result<bool, openssl::error::ErrorStack> {
    let mut verifier = openssl::sign::Verifier::new(alg, &key)?;
    verifier.update(&src)?;
    Ok(verifier.verify(sig)?)
}

pub fn must_authenticate(actor: Actor, body: Activity) -> Result<(), warp::Rejection>{
    if actor.uri.as_str() == body.actor.as_str() {
        log::info!("must_authenticate verified");
        Ok(())
    } else {
        log::info!("must_authenticate failed: {}, {}", actor.uri.as_str(), body.actor.as_str());
        Err(warp::reject())
    }
}

pub async fn authenticate_http_signatures(
    app_state: Arc<AppState>,
    _domain: String,
    method: Method,
    full_path: FullPath,
    headers: HeaderMap
) -> Result<Actor, warp::Rejection> {
    let signature = headers.get("Signature").ok_or(warp::reject())?;

    let signature = hancock::Signature::parse(&signature).or(Err(warp::reject()))?;

    let actor_key_id = signature.key_id.clone().ok_or(warp::reject())?;

    let actor = if actor_key_id.ends_with("#main-key") {
        let actor_id = String::from(actor_key_id.split("#main-key").collect::<Vec<&str>>()[0]);
        apub::actions::get_or_fetch_actor_by_uri(&app_state.db, actor_id.as_str()).await.ok()   
    } else {
        None
    };

    let actor = actor.ok_or(warp::reject())?;

    let key = openssl::pkey::PKey::public_key_from_pem(actor.public_key_pem.as_bytes()).or(Err(warp::reject()))?;

    let verified = signature
        .verify(
            &method,
            full_path.as_str(),
            &headers,
            |body: &[u8], signature: &[u8]| {
                do_verify(
                    &key,
                    openssl::hash::MessageDigest::sha256(),
                    body,
                    signature,
                )
            },
        )
        .unwrap_or(false);
    if verified {
        Ok(actor)
    } else {
        Err(warp::reject())
    }
}
