use pretty_assertions::assert_eq;
use tonic::Request;

use cycling_tracker::cycling_tracker::{WorkoutPlan, WorkoutPlanToken};
use crate::common::run_service_and_get_client;

#[tokio::test]
async fn test_create_workout_plan() {
    let mut client = run_service_and_get_client().await;

    let request = Request::new(WorkoutPlan { steps: vec![] });

    let actual_response = client
        .create_workout_plan(request)
        .await
        .expect("Failed to create workout plan")
        .into_inner();

    let expect_response = WorkoutPlanToken { workout_token: 0 };

    assert_eq!(actual_response, expect_response);
}
