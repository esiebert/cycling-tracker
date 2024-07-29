use crate::cycling_tracker::{Measurement, Workout, WorkoutSummary};
use crate::handler::database::SQLiteHandler;

#[derive(Clone)]
pub struct WorkoutHandler {
    pub sqlite_handler: SQLiteHandler,
}

impl WorkoutHandler {
    pub async fn save_workout(&self, workout: &Workout) -> WorkoutSummary {
        let mut summary = self.create_summary(workout);
        let summary_id = self.sqlite_handler.save_workout(&summary).await;
        summary.id = Some(summary_id);

        summary
    }

    pub fn create_summary(&self, workout: &Workout) -> WorkoutSummary {
        let readings = workout.measurements.len();

        if readings == 0 {
            return WorkoutSummary {
                km_ridden: 0.0,
                ..Default::default()
            };
        }

        let acc_measurements = workout
            .measurements
            .clone()
            .into_iter()
            .reduce(|acc, e| acc + e)
            .unwrap();

        WorkoutSummary {
            id: None,
            km_ridden: workout.km_ridden,
            avg_speed: acc_measurements.speed / readings as f32,
            avg_watts: acc_measurements.watts / readings as i32,
            avg_rpm: acc_measurements.rpm / readings as i32,
            avg_heartrate: acc_measurements.heartrate / readings as i32,
            measurements: workout.measurements.clone(),
        }
    }

    pub async fn get_measurements(&self, workout_id: i32) -> Option<Vec<Measurement>> {
        self.sqlite_handler.get_measurements(workout_id).await
    }
}
