use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use thiserror::Error;

use crate::cycling_tracker::{
    Measurement, WorkoutPlan, WorkoutPlanToken, WorkoutSummary,
};

#[derive(Clone)]
pub struct SQLiteHandler {
    db: SqlitePool,
}

impl SQLiteHandler {
    pub async fn new(db_url: &str) -> Result<Self, DatabaseError> {
        Sqlite::create_database(db_url)
            .await
            .map_err(|e| DatabaseError::CreationFailed(format!("{e:?}")))?;

        let db = SqlitePool::connect(db_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(format!("{e:?}")))?;

        sqlx::migrate!()
            .run(&db)
            .await
            .map_err(|e| DatabaseError::MigrationFailed(format!("{e:?}")))?;

        Ok(Self { db })
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
                "INSERT INTO MEASUREMENTS VALUES ($1, $2, $3, $4, $5, $6)",
                measurement.speed,
                measurement.watts,
                measurement.rpm,
                measurement.resistance,
                measurement.heartrate,
                summary_id,
            )
            .execute(&self.db)
            .await;
        }

        println!("Saved summary to database");

        summary_id as i32
    }

    pub async fn save_plan(&self, plan: &WorkoutPlan) -> WorkoutPlanToken {
        println!("Saving to database = {:?}", plan);
        WorkoutPlanToken { workout_token: 1 }
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
                resistance: r.resistance as i32,
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
