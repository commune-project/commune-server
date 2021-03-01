use crate::db::schema::users;

use validator::Validate;

#[derive(Clone, Identifiable, Queryable, Insertable, Associations, PartialEq, Debug, Default, Validate)]
#[belongs_to(super::Actor)]
#[table_name = "users"]
#[primary_key(actor_id)]
pub struct User {
    pub actor_id: i64,
    #[validate(email)]
    pub email: Option<String>,
    pub is_email_verified: bool,
    pub password_hash: Option<String>,
    pub private_key_pem: String,
    pub register_ip: Option<String>,
    pub last_login_ip: Option<String>,
}
