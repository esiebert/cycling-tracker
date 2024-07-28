use crate::cycling_tracker::{
    Measurement, WorkoutPlan, WorkoutPlanToken, WorkoutSummary,
};

#[derive(Clone)]
pub struct SQLiteHandler {}

impl SQLiteHandler {
    pub async fn save_workout(&self, summary: &WorkoutSummary) {
        println!("Saving to database = {:?}", summary);
    }

    pub async fn save_plan(&self, plan: &WorkoutPlan) -> WorkoutPlanToken {
        println!("Saving to database = {:?}", plan);
        WorkoutPlanToken { workout_token: 1 }
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
