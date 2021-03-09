use crate::fixtures::create_user_fixture;

use diesel::Connection;

use commune::db::establish_connection;
use commune::db::actions::actor;
use commune::errors::{ActionResult, ActionError};

#[test]
fn test_get_actor_by_username_domain() -> ActionResult<()> {
    let conn = establish_connection();
    conn.begin_test_transaction().map_err(|_e| ActionError::InternalError)?;
    let user_actor = create_user_fixture(&conn, "misaka4e22", "test1.example.tld");    
    let actor = actor::get_actor_by_username_domain(&conn, "misaka4e22", "test1.example.tld")?;    
    assert_eq!(actor, user_actor.actor);
    Ok(())
}


#[test]
fn test_get_actor_by_uri() -> ActionResult<()> {
    let conn = establish_connection();
    conn.begin_test_transaction().map_err(|_e| ActionError::InternalError)?;
    let user_actor = create_user_fixture(&conn, "misaka4e22", "test1.example.tld");    
    let actor = actor::get_actor_by_uri(&conn, "https://test1.example.tld/users/misaka4e22")?;    
    assert_eq!(actor, user_actor.actor);
    Ok(())
}

#[test]
fn test_insert_new_actor() -> ActionResult<()> {
    use commune::db::models::NewActor;
    use chrono::Utc;
    let conn = establish_connection();
    conn.begin_test_transaction().map_err(|_e| ActionError::InternalError)?;
    let now = Utc::now().naive_utc();
    let new_actor = NewActor {
        kind: String::from("Person"),
        username: String::from("misaka4e23"),
        domain: String::from("test2.example.tld"),
        uri: String::from("https://test2.example.tld/users/misaka4e23"),
        url: Some(String::from("https://test2.example.tld/users/misaka4e23")),
        inbox_uri: String::from("https://test2.example.tld/users/misaka4e23/inbox"),
        outbox_uri: String::from("https://test2.example.tld/users/misaka4e23/outbox"),
        followers_uri: Some(String::from("https://test2.example.tld/users/misaka4e23/followers")),
        following_uri: Some(String::from("https://test2.example.tld/users/misaka4e23/following")),
        created_at: Some(now),
        updated_at: Some(now),
        public_key_pem: String::from("TEST_CERT"),

        ..Default::default()
    };
    let actor = actor::insert_new_actor(&conn, new_actor)?;
    let actor_get = actor::get_actor_by_uri(&conn, "https://test2.example.tld/users/misaka4e23")?;    
    assert_eq!(actor, actor_get);
    Ok(())
}