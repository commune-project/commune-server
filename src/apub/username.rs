use idna::{domain_to_ascii, domain_to_unicode};

pub fn username_to_idna(username: &str) -> String {
    match domain_to_ascii(username) {
        Ok(s) => s,
        Err(_e) => String::from(username),
    }
}

pub fn idna_to_username(idna: &str) -> String {
    let (username, _result) = domain_to_unicode(idna);
    username
}
