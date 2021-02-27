use serde_json::Value;

pub fn get_uri(object: Value) -> Option<String> {
    match object {
        String(s) => Some(s),
        Object(_map) => object.get("id"),
        _ => None
    }
}