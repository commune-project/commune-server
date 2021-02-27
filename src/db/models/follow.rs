use crate::db::schema::follows;
use chrono;

#[derive(Clone, Identifiable, Queryable, Insertable, Associations, PartialEq, Debug)]
#[primary_key(follower_id, following_id)]
#[table_name = "follows"]
pub struct Follow {
    pub follower_id: i64,
    pub following_id: i64,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    
    pub role: String,
}