use diesel::r2d2::ConnectionManager;
use diesel::pg::PgConnection;
use dotenv;
use std::env;
use r2d2;
use std::collections::HashSet;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// This struct represents state
#[derive(Clone)]
pub struct AppState {
    pub website_name: String,
    pub db: DbPool,
    pub local_domains: HashSet<String>,
}

impl AppState {
    pub(crate) fn new() -> AppState {
        dotenv::dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set!");
        let local_domains_env = env::var("LOCAL_DOMAINS").expect("LOCAL_DOMAINS not set!");
        let local_domains = local_domains_env.split_ascii_whitespace();

        let mut local_domain_set = HashSet::new();
        for local_domain in local_domains {
            local_domain_set.insert(String::from(local_domain));
        }

        // Create connection pool
        let pool: DbPool = r2d2::Pool::builder().max_size(1).build(ConnectionManager::new(db_url)).expect("Failed to create pool.");

        AppState {
            website_name: String::from("Commune"),
            db: pool,
            local_domains: local_domain_set,
        }
    }
}
