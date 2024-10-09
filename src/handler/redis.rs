use redis::Commands;

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

        if let Some(expiry) = expiry {
            con.set_ex::<String, String, ()>(
                key.to_string(),
                value.to_string(),
                expiry,
            )
            .expect("Failed to set Redis key");
        } else {
            con.set::<String, String, ()>(key.to_string(), value.to_string())
                .expect("Failed to set Redis key");
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
