use sqlx::SqlitePool;
use thiserror::Error;

use crate::cycling_tracker::{Measurement, WorkoutSummary};

#[derive(Clone)]
pub struct SQLiteHandler {
    pub db: SqlitePool,
}

impl SQLiteHandler {
    pub async fn create_user(&self, username: String, password: String) -> bool {
        match sqlx::query!("INSERT INTO USER VALUES ($1, $2)", username, password)
            .execute(&self.db)
            .await
        {
            Ok(_) => true,
            Err(e) => {
                println!("Failed to create user: {:?}", e);
                false
            }
        }
    }

    pub async fn get_hashed_password(&self, username: String) -> Option<String> {
        match sqlx::query!("SELECT password FROM USER WHERE username = $1", username)
            .fetch_one(&self.db)
            .await
        {
            Ok(record) => Some(record.password),
            Err(_) => None,
        }
    }

    pub async fn save_workout(&self, summary: &WorkoutSummary) -> i32 {
        let result = sqlx::query!(
            "INSERT INTO WORKOUT_SUMMARY VALUES (Null, $1, $2, $3, $4, $5)",
            summary.km_ridden,
            summary.avg_speed,
            summary.avg_watts,
            summary.avg_rpm,
            summary.avg_heartrate,
        )
        .execute(&self.db)
        .await;

        let summary_id = result.unwrap().last_insert_rowid();

        for measurement in summary.measurements.clone() {
            let _ = sqlx::query!(
                "INSERT INTO MEASUREMENTS VALUES ($1, $2, $3, $4, $5)",
                measurement.speed,
                measurement.watts,
                measurement.rpm,
                measurement.heartrate,
                summary_id,
            )
            .execute(&self.db)
            .await;
        }

        println!("Saved summary to database");

        summary_id as i32
    }

    pub async fn get_measurements(&self, workout_id: i32) -> Option<Vec<Measurement>> {
        // We can't use query_as, because the db fields are 64 bits by default,
        // and therefore we have to cast the values by hand
        let records = sqlx::query!(
            "SELECT * FROM MEASUREMENTS WHERE workout_id = $1",
            workout_id
        )
        .fetch_all(&self.db)
        .await
        .unwrap();

        let measurements = records
            .iter()
            .map(|r| Measurement {
                speed: r.speed as f32,
                watts: r.watts as i32,
                rpm: r.rpm as i32,
                heartrate: r.heartrate as i32,
            })
            .collect();

        Some(measurements)
    }
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to create database: {0}")]
    CreationFailed(String),

    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(String),

    #[error("Failed to migrate database: {0}")]
    MigrationFailed(String),
}
