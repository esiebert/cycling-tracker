use anyhow::Result;
use tokio::sync::mpsc::{channel, Receiver};

use crate::handler::database;

pub struct SQLite {
    receiver: Receiver<database::Message>,
    handler: database::SQLiteHandler,
}

impl SQLite {
    pub fn new() -> Self {
        let (sender, receiver) = channel(32);

        Self {
            receiver,
            handler: database::SQLiteHandler { sender },
        }
    }

    pub async fn run(mut self) -> Result<()> {
        while let Some(message) = self.receiver.recv().await {
            self.handler.handle_message(message).await;
        }
        Ok(())
    }

    pub fn handler(&self) -> database::SQLiteHandler {
        self.handler.clone()
    }
}

impl Default for SQLite {
    fn default() -> Self {
        Self::new()
    }
}
