pub mod actor;
pub mod image;

pub use actor::ActorSerializer;
pub use image::ImageSerializer;

use serde_json::{
    json,
    value::Value
};

pub fn get_context() -> Value {
    json!([
        "https://www.w3.org/ns/activitystreams",
        "https://litepub.social/context.jsonld"
    ])
}