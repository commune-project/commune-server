use diesel::PgConnection;
use commune::db::models::UserActor;
use commune::db::actions;

const COMMON_PASSWORD: &str = "123456";

pub fn create_user_fixture(
    conn: &PgConnection,
    username: &str,
    domain: &str
) -> UserActor {
    let email: String = format!("{}@{}.example.tld", username, domain);
    match actions::user::create_user(conn, username, domain, COMMON_PASSWORD, email.as_str(), "und", None) {
        Ok(useractor) => useractor,
        Err(e) => panic!("error: {}", e)
    }
}