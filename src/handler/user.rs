use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

use crate::cycling_tracker::Credentials;
use crate::handler::SQLiteHandler;

#[derive(Clone)]
pub struct UserHandler {
    pub sqlite_handler: SQLiteHandler,
}

impl UserHandler {
    pub async fn create(&self, credentials: Credentials) -> bool {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(credentials.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        self.sqlite_handler
            .create_user(credentials.username, hash)
            .await
    }

    pub async fn login(&self, credentials: Credentials) -> bool {
        let password_hash = self
            .sqlite_handler
            .get_hashed_password(credentials.username)
            .await;
        match password_hash {
            Some(hash) => {
                let parsed_hash = PasswordHash::new(&hash).unwrap();
                return Argon2::default()
                    .verify_password(credentials.password.as_bytes(), &parsed_hash)
                    .is_ok();
            }
            None => false,
        }
    }
}
