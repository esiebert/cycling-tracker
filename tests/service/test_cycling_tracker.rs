use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tonic::Request;

use crate::common::{run_test_env, stream_to_vec, vec_to_stream};
use cycling_tracker::cycling_tracker::{
    Measurement, Workout, WorkoutRequest, WorkoutSummary,
};

lazy_static! {
    static ref MEASUREMENTS: Vec<Measurement> = vec![
        Measurement {
            speed: 29.0,
            watts: 290,
            rpm: 90,
            heartrate: 130,
        },
        Measurement {
            speed: 30.0,
            watts: 300,
            rpm: 95,
            heartrate: 140,
        },
        Measurement {
            speed: 31.0,
            watts: 310,
            rpm: 100,
            heartrate: 150,
        },
    ];
    static ref WORKOUT_SUMMARY: WorkoutSummary = WorkoutSummary {
        id: Some(1),
        km_ridden: 53.5,
        avg_speed: 30.0,
        avg_watts: 300,
        avg_rpm: 95,
        avg_heartrate: 140,
        measurements: (*MEASUREMENTS).clone(),
    };
}

#[sqlx::test]
async fn test_save_workout_and_get_measurements(db: SqlitePool) {
    let mut test_env = run_test_env(db).await;

    let save_request = Request::new(Workout {
        km_ridden: 53.5,
        measurements: (*MEASUREMENTS).clone(),
    });

    let actual_response = test_env
        .grpc_client
        .save_workout(save_request)
        .await
        .expect("Failed to save workout")
        .into_inner();

    assert_eq!(actual_response, *WORKOUT_SUMMARY);

    let get_request = Request::new(WorkoutRequest { id: 1 });

    let response_stream = test_env
        .grpc_client
        .get_measurements(get_request)
        .await
        .expect("Failed to get measurements")
        .into_inner();

    let actual_response_stream = stream_to_vec(response_stream).await;
    assert_eq!(actual_response_stream, (*MEASUREMENTS).clone());
}

#[sqlx::test]
async fn test_record_workout(db: SqlitePool) {
    let mut test_env = run_test_env(db).await;

    let request = vec_to_stream((*MEASUREMENTS).clone());

    let actual_response = test_env
        .grpc_client
        .record_workout(request)
        .await
        .expect("Failed to record workout")
        .into_inner();

    assert_eq!(actual_response, *WORKOUT_SUMMARY);
}

#[sqlx::test]
async fn test_get_current_averages(db: SqlitePool) {
    let mut test_env = run_test_env(db).await;

    let request = vec_to_stream((*MEASUREMENTS).clone());

    let response_stream = test_env
        .grpc_client
        .get_current_averages(request)
        .await
        .expect("Failed to get current averages")
        .into_inner();

    let expected_response = vec![
        WorkoutSummary {
            id: None,
            km_ridden: 29.0,
            avg_speed: 29.0,
            avg_watts: 290,
            avg_rpm: 90,
            avg_heartrate: 130,
            measurements: vec![Measurement {
                speed: 29.0,
                watts: 290,
                rpm: 90,
                heartrate: 130,
            }],
        },
        WorkoutSummary {
            id: None,
            km_ridden: 59.0,
            avg_speed: 29.5,
            avg_watts: 295,
            avg_rpm: 92,
            avg_heartrate: 135,
            measurements: vec![
                Measurement {
                    speed: 29.0,
                    watts: 290,
                    rpm: 90,
                    heartrate: 130,
                },
                Measurement {
                    speed: 30.0,
                    watts: 300,
                    rpm: 95,
                    heartrate: 140,
                },
            ],
        },
        WorkoutSummary {
            id: None,
            km_ridden: 90.0,
            avg_speed: 30.0,
            avg_watts: 300,
            avg_rpm: 95,
            avg_heartrate: 140,
            measurements: vec![
                Measurement {
                    speed: 29.0,
                    watts: 290,
                    rpm: 90,
                    heartrate: 130,
                },
                Measurement {
                    speed: 30.0,
                    watts: 300,
                    rpm: 95,
                    heartrate: 140,
                },
                Measurement {
                    speed: 31.0,
                    watts: 310,
                    rpm: 100,
                    heartrate: 150,
                },
            ],
        },
    ];

    let actual_response = stream_to_vec(response_stream).await;
    assert_eq!(actual_response, expected_response);
}
