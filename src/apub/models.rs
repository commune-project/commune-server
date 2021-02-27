pub mod activities;
pub mod actor;

pub use activities::*;
pub use actor::*;

pub fn empty_string_or_none(value: String) -> Option<String> {
    if value == "" {
        None
    } else {
        Some(value)
    }
}