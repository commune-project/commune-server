use crate::db::actions::actor::get_actor_by_uri;
use crate::db::models::{Actor, Follow};
use crate::db::schema;
use crate::errors::{ActionError, ActionResult};
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;

pub const PAGE_SIZE: i64 = 12;

pub fn actor_get_followers(
    db: &PgConnection,
    actor: &Actor,
    page: i64,
) -> ActionResult<Vec<Actor>> {
    use schema::actors;
    use schema::follows;
    let data = actors::table
        .inner_join(
            follows::table.on(follows::follower_id
                .eq(actors::id)
                .and(follows::following_id.eq(actor.id))
                .and(follows::role.ne("pending"))),
        )
        .order(follows::created_at.desc())
        .limit(PAGE_SIZE)
        .offset(PAGE_SIZE * (page - 1))
        .load(db);
    data.map(|v: Vec<(Actor, Follow)>| v.into_iter().map(|(a, _f)| a).collect())
        .map_err(|_e| ActionError::NotFound)
}

pub fn actor_count_followers(db: &PgConnection, actor: &Actor) -> ActionResult<i64> {
    use diesel::dsl::count_star;
    use schema::actors;
    use schema::follows;
    let data = actors::table
        .inner_join(
            follows::table.on(follows::follower_id
                .eq(actors::id)
                .and(follows::following_id.eq(actor.id))
                .and(follows::role.ne("pending"))),
        )
        .select(count_star())
        .first(db);
    data.map_err(|_e| ActionError::NotFound)
}

pub fn follow_actor_by_uri(
    db: &PgConnection,
    follower_uri: &str,
    following_uri: &str,
) -> ActionResult<Follow> {
    let follower_actor = get_actor_by_uri(db, follower_uri)?;
    let following_actor = get_actor_by_uri(db, following_uri)?;
    let now = Utc::now().naive_utc();
    let new_follow = Follow {
        follower_id: follower_actor.id,
        following_id: following_actor.id,
        created_at: now,
        updated_at: Some(now),
        role: if following_actor.is_locked {
            String::from("pending")
        } else {
            String::from("follower")
        },
    };

    diesel::insert_into(schema::follows::table)
        .values(&new_follow)
        .get_result::<Follow>(db)
        .map_err(|_e| ActionError::InsertError)
}

pub fn unfollow_actor_by_uri(
    db: &PgConnection,
    follower_uri: &str,
    following_uri: &str,
) -> ActionResult<()> {
    use schema::follows::dsl::*;
    let follower_actor = get_actor_by_uri(db, follower_uri)?;
    let following_actor = get_actor_by_uri(db, following_uri)?;
    diesel::delete(
        follows.filter(
            follower_id
                .eq(follower_actor.id)
                .and(following_id.eq(following_actor.id)),
        ),
    )
    .execute(db)
    .map(|_v| ())
    .map_err(|_e| ActionError::NotFound)
}
