pub mod actor;
pub mod user;
pub mod follow;

pub use actor::*;
pub use user::*;
pub use follow::*;

#[derive(Clone, PartialEq, Debug)]
pub struct UserActor {
    pub actor: Actor,
    pub user: User,
}