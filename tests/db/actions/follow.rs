use crate::fixtures::create_user_fixture;

use diesel::Connection;

use commune::db::establish_connection;
use commune::db::actions::follow::{follow_actor_by_uri, actor_get_followers};
use commune::errors::{ActionResult, ActionError};

#[test]
fn test_follow_actor_by_uri() -> ActionResult<()> {
    let conn = establish_connection();
    conn.begin_test_transaction().map_err(|_e| ActionError::InternalError)?;
    let user_actor1 = create_user_fixture(&conn, "misaka4e21", "test2.example.tld");
    let user_actor2 = create_user_fixture(&conn, "misaka4e22", "test1.example.tld");
    let follow = follow_actor_by_uri(&conn, "https://test2.example.tld/users/misaka4e21", "https://test1.example.tld/users/misaka4e22")?;
    assert_eq!(follow.follower_id, user_actor1.actor.id);
    assert_eq!(follow.following_id, user_actor2.actor.id);
    Ok(())
}

#[test]
fn test_actor_get_followers() -> ActionResult<()> {
    let conn = establish_connection();
    conn.begin_test_transaction().map_err(|_e| ActionError::InternalError)?;
    let user_actor1 = create_user_fixture(&conn, "misaka4e21", "test1.example.tld");
    let user_actor2 = create_user_fixture(&conn, "misaka4e22", "test1.example.tld");
    let _follow = follow_actor_by_uri(&conn, "https://test1.example.tld/users/misaka4e21", "https://test1.example.tld/users/misaka4e22")?;

    let followers_vec = actor_get_followers(&conn, &user_actor2.actor, 1)?;
    assert_eq!(followers_vec[0], user_actor1.actor);

    Ok(())
}