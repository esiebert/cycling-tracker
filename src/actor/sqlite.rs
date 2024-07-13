use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct SQLiteActor {
    receiver: Receiver<Message>,
    handler: Sender<Message>,
}

#[derive(Debug)]
pub enum Message {
    SaveWorkout(String),
}

impl SQLiteActor {
    pub fn new() -> Self {
        let (handler, receiver) = channel(32);
        Self {
            receiver: receiver,
            handler: handler,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(i) = self.receiver.recv().await {
            println!("got = {:?}", i);
        }
        Ok(())
    }

    pub fn handler(&self) -> Sender<Message> {
        self.handler.clone()
    }
}
