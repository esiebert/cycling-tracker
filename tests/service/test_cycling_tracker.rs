use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use tonic::Request;

use crate::common::{run_test_env, stream_to_vec, vec_to_stream};
use cycling_tracker::cycling_tracker::{
    ControlStep, Measurement, StepType, Workout, WorkoutPlan, WorkoutPlanToken,
    WorkoutRequest, WorkoutStep, WorkoutSummary,
};

lazy_static! {
    static ref MEASUREMENTS: Vec<Measurement> = vec![
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
    ];
    static ref WORKOUT_SUMMARY: WorkoutSummary = WorkoutSummary {
        id: 1,
        km_ridden: 53.5,
        avg_speed: 30.0,
        avg_watts: 300,
        avg_rpm: 95,
        avg_heartrate: 140,
        measurements: (*MEASUREMENTS).clone(),
    };
}

#[tokio::test]
async fn test_save_workout() {
    let mut test_env = run_test_env().await;

    let request = Request::new(Workout {
        km_ridden: 53.5,
        measurements: (*MEASUREMENTS).clone(),
    });

    let actual_response = test_env
        .grpc_client
        .save_workout(request)
        .await
        .expect("Failed to save workout")
        .into_inner();

    assert_eq!(actual_response, *WORKOUT_SUMMARY);
}

#[tokio::test]
async fn test_get_measurements() {
    let mut test_env = run_test_env().await;

    let request = Request::new(WorkoutRequest { id: 1 });

    let response_stream = test_env
        .grpc_client
        .get_measurements(request)
        .await
        .expect("Failed to get measurements")
        .into_inner();

    let actual_response = stream_to_vec(response_stream).await;
    assert_eq!(actual_response, (*MEASUREMENTS).clone());
}

#[tokio::test]
async fn test_record_workout() {
    let mut test_env = run_test_env().await;

    let request = vec_to_stream((*MEASUREMENTS).clone());

    let actual_response = test_env
        .grpc_client
        .record_workout(request)
        .await
        .expect("Failed to record workout")
        .into_inner();

    assert_eq!(actual_response, *WORKOUT_SUMMARY);
}

#[tokio::test]
async fn test_run_workout() {
    let mut test_env = run_test_env().await;

    let request = vec_to_stream(vec![
        WorkoutStep {
            stype: StepType::Starting.into(),
            ..Default::default()
        },
        WorkoutStep {
            stype: StepType::InProgress.into(),
            ..Default::default()
        },
        WorkoutStep {
            stype: StepType::Ending.into(),
            ..Default::default()
        },
    ]);

    let response_stream = test_env
        .grpc_client
        .run_workout(request)
        .await
        .expect("Failed to run workout")
        .into_inner();

    let expected_response = vec![
        ControlStep {
            stype: StepType::Starting.into(),
            resistance: Some(150),
            workout_summary_id: None,
        },
        ControlStep {
            stype: StepType::InProgress.into(),
            resistance: Some(200),
            workout_summary_id: None,
        },
        ControlStep {
            stype: StepType::Ending.into(),
            resistance: None,
            workout_summary_id: Some(1),
        },
    ];

    let actual_response = stream_to_vec(response_stream).await;
    assert_eq!(actual_response, expected_response);
}

#[tokio::test]
async fn test_create_workout_plan() {
    let mut test_env = run_test_env().await;

    let request = Request::new(WorkoutPlan { steps: vec![] });

    let actual_response = test_env
        .grpc_client
        .create_workout_plan(request)
        .await
        .expect("Failed to create workout plan")
        .into_inner();

    let expect_response = WorkoutPlanToken { workout_token: 0 };

    assert_eq!(actual_response, expect_response);
}
