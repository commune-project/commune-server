use openssl::rsa::Rsa;

pub struct KeyPair {
    pub public: String,
    pub private: String,
}

pub fn generate_key_pair_pem() -> Option<KeyPair> {
    let rsa = Rsa::generate(2048);
    match rsa {
        Err(_) => None,
        Ok(key) => {
            let private_key = match key.private_key_to_pem() {
                Ok(vu8) => String::from_utf8(vu8).ok(),
                Err(_) => None
            };
            let public_key = match key.public_key_to_pem() {
                Ok(vu8) => String::from_utf8(vu8).ok(),
                Err(_) => None
            };
            if let Some(public) = public_key {
                if let Some(private) = private_key {
                    return Some(KeyPair{
                        public,
                        private
                    });
                }
            }
            None
        }
    }
}