use tokio::sync::mpsc::Sender;

use crate::cycling_tracker::Measurement;

#[derive(Debug)]
pub enum Message {
    SaveWorkout(String),
}

#[derive(Clone)]
pub struct SQLiteHandler {
    pub sender: Sender<Message>,
}

impl SQLiteHandler {
    pub async fn send(&self, message: Message) {
        let _ = self.sender.send(message).await;
    }

    pub async fn handle_message(&self, message: Message) {
        println!("Saving message to database = {:?}", message);
    }

    pub async fn get_measurements(&self, _workout_id: i32) -> Option<Vec<Measurement>> {
        Some(vec![
            Measurement {
                speed: 29.0,
                watts: 290,
                rpm: 90,
                resistance: 690,
                heartrate: 130,
            },
            Measurement {
                speed: 30.0,
                watts: 300,
                rpm: 95,
                resistance: 700,
                heartrate: 140,
            },
            Measurement {
                speed: 31.0,
                watts: 310,
                rpm: 100,
                resistance: 710,
                heartrate: 150,
            },
        ])
    }
}
