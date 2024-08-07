use redis::{Commands, ExistenceCheck, SetExpiry, SetOptions};

#[derive(Clone)]
pub struct RedisHandler {
    pub client: redis::Client,
}

impl RedisHandler {
    pub fn set_key(&self, key: &String, value: &String, expiry: Option<u64>) {
        let mut con = self
            .client
            .get_connection()
            .expect("Failed to open connection with redis");

        if expiry.is_some() {
            let opts = SetOptions::default()
                .conditional_set(ExistenceCheck::NX)
                .get(true)
                .with_expiration(SetExpiry::EX(expiry.unwrap()));
            con.set_options::<String, String, ()>(
                key.to_string(),
                value.to_string(),
                opts,
            )
            .unwrap();
        } else {
            con.set::<String, String, ()>(key.to_string(), value.to_string())
                .unwrap();
        }
    }

    pub fn get_key(&self, key: &str) -> Option<String> {
        let mut con = self
            .client
            .get_connection()
            .expect("Failed to open connection with redis");

        match con.get(key) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}
