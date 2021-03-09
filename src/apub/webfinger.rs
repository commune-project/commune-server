use crate::apub::username::{idna_to_username, username_to_idna};
use crate::db::actions::actor::{get_actor_by_uri, get_actor_by_username_domain};
use crate::errors;
use diesel::PgConnection;
use reqwest;
use serde::Deserialize;
use serde_json::{json, Value};
use url;

#[derive(Clone)]
pub struct WebfingerInfo {
    pub acct: WebfingerAcct,
    pub ap: WebfingerAP,
}

#[derive(Clone)]
pub struct WebfingerAP {
    pub uri: String,
    pub url: String,
}

#[derive(Clone)]
pub struct WebfingerAcct {
    pub preferred_username: String,
    pub domain: String,
}

#[derive(Deserialize)]
struct WebfingerResponse {
    pub subject: String,
    pub links: Vec<Link>,
}

#[derive(Deserialize)]
struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub href: Option<String>,
}

pub async fn query_webfinger(resource: String) -> errors::ActionResult<WebfingerInfo> {
    let webfinger_url = format!(
        "https://{}/.well-known/webfinger?resource={}",
        get_domain(resource.clone()).ok_or(errors::ActionError::FetchError)?,
        resource.clone()
    );
    let response = reqwest::get(webfinger_url.as_str())
        .await
        .map_err(|_e| errors::ActionError::FetchError)?
        .json::<WebfingerResponse>()
        .await
        .map_err(|_e| errors::ActionError::FetchError)?;
    let mut uri = String::from("");
    let mut url = String::from("");
    for link in &response.links {
        if link.kind == Some(String::from("application/activity+json")) {
            uri = link.href.clone().unwrap_or(String::from(""));
        } else if link.rel == String::from("http://webfinger.net/rel/profile-page") {
            url = link.href.clone().unwrap_or(String::from(""));
        }
    }
    let acct = parse_acct(response.subject).ok_or(errors::ActionError::FetchError)?;
    if uri != "" && url != "" {
        Ok(WebfingerInfo {
            acct,
            ap: WebfingerAP { uri, url },
        })
    } else {
        Err(errors::ActionError::FetchError)
    }
}

fn get_domain(resource: String) -> Option<String> {
    if resource.starts_with("acct:") {
        parse_acct(resource).map(|wa| wa.domain)
    } else {
        match url::Url::parse(resource.as_str()) {
            Ok(u) => u.host_str().map(|s| String::from(s)),
            Err(_) => None,
        }
    }
}

fn parse_acct(subject: String) -> Option<WebfingerAcct> {
    match subject.get(5..subject.len()) {
        Some(s) => {
            let v: Vec<&str> = s.split("@").collect();
            if v.len() == 2 {
                Some(WebfingerAcct {
                    preferred_username: String::from(v[0]),
                    domain: String::from(v[1]),
                })
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn get_webfinger(db: &PgConnection, resource: &str) -> errors::ActionResult<Value> {
    let actor = match parse_acct(String::from(resource)) {
        Some(webfinger_acct) => {
            let username = idna_to_username(webfinger_acct.preferred_username.as_str());
            let domain = webfinger_acct.domain;
            get_actor_by_username_domain(db, username.as_str(), domain.as_str())?
        }
        None => get_actor_by_uri(db, resource)?,
    };

    Ok(json!({
        "subject": format!("acct:{}@{}", username_to_idna(actor.username.as_str()), actor.domain),
        "aliases":[actor.uri, actor.url],
        "links":[
            {
                "rel": "http://webfinger.net/rel/profile-page",
                "type": "text/html",
                "href": actor.url
            },
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": actor.uri
            }
        ]
    }))
}