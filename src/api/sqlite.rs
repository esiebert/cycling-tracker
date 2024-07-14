use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct SQLite {
    receiver: Receiver<Message>,
    handler: Sender<Message>,
}

#[derive(Debug)]
pub enum Message {
    SaveWorkout(String),
}

impl SQLite {
    pub fn new() -> Self {
        let (handler, receiver) = channel(32);
        Self {
            receiver: receiver,
            handler: handler,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(i) = self.receiver.recv().await {
            println!("Saving log to database = {:?}", i);
        }
        Ok(())
    }

    pub fn handler(&self) -> Sender<Message> {
        self.handler.clone()
    }
}
