use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::prelude::*;
use diesel::PgConnection;

use crate::apub;
use crate::db::schema;

use crate::db::models::actor::{Actor, ActorType, NewActor, NewLocalActorBuilder};
use crate::db::models::{User, UserActor};
use crate::errors::{ActionError, ActionResult};

use bcrypt;

pub fn create_user(
    conn: &PgConnection,
    username: &str,
    domain: &str,
    password: &str,
    email: &str,
    lang: &str,
    register_ip: Option<String>,
) -> ActionResult<UserActor> {
    let password_hash = match bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(err) => return Err(ActionError::InternalError),
    };

    let keypair = match apub::rsa::generate_key_pair_pem() {
        Some(pair) => pair,
        None => return Err(ActionError::InternalError),
    };

    let new_actor = NewLocalActorBuilder {
        username,
        domain,
        lang,
        actor_type: ActorType::Person,
        public_key_pem: keypair.public.as_str(),
    }
    .build();

    let tm = AnsiTransactionManager::new();

    tm.begin_transaction(conn)
        .map_err(|e| ActionError::InsertError)?;

    let actor = diesel::insert_into(schema::actors::table)
        .values(&new_actor)
        .get_result::<Actor>(conn)
        .map_err(|_| ActionError::InsertError)?;
    let email = match email {
        "" => None,
        _ => Some(String::from(email)),
    };
    let new_user = User {
        actor_id: actor.id,
        email,
        is_email_verified: false,
        password_hash: Some(password_hash),
        private_key_pem: keypair.private,
        register_ip,
        ..Default::default()
    };

    let user = diesel::insert_into(schema::users::table)
        .values(&new_user)
        .get_result::<User>(conn)
        .map_err(|_| ActionError::InsertError)?;

    tm.commit_transaction(conn)
        .map_err(|_| ActionError::InsertError)?;

    Ok(UserActor { actor, user })
}
