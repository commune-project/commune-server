use serde_json::{
    json,
    value::Value
};

pub struct ImageSerializer {
    image_url: String
}

impl ImageSerializer {
    pub fn new(image_url: &str) -> Self {
        ImageSerializer {
            image_url: String::from(image_url),
        }
    }


    pub fn serialize(self) -> Value {
        json!({
            "type": "Image",
            "url": self.image_url
        })
    }
}