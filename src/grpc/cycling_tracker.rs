use std::collections::VecDeque;
use std::pin::Pin;

use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use crate::cycling_tracker::cycling_tracker_server::CyclingTracker;
use crate::cycling_tracker::{
    workout_plan::Step, ControlStep, Measurement, StepType, Workout, WorkoutPlan,
    WorkoutPlanToken, WorkoutRequest, WorkoutStep, WorkoutSummary,
};
use crate::handler::WorkoutHandler;

type GRPCResult<T> = Result<Response<T>, Status>;

#[derive(Clone)]
pub struct CyclingTrackerService {
    workout_handler: WorkoutHandler,
}

impl CyclingTrackerService {
    pub fn new(workout_handler: WorkoutHandler) -> Self {
        Self { workout_handler }
    }
}

#[tonic::async_trait]
impl CyclingTracker for CyclingTrackerService {
    async fn save_workout(
        &self,
        request: Request<Workout>,
    ) -> GRPCResult<WorkoutSummary> {
        let workout = request.into_inner();

        let summary = self.workout_handler.save_workout(&workout).await;

        Ok(Response::new(summary))
    }

    type GetMeasurementsStream = ReceiverStream<Result<Measurement, Status>>;

    async fn get_measurements(
        &self,
        request: Request<WorkoutRequest>,
    ) -> GRPCResult<Self::GetMeasurementsStream> {
        let workout_id = request.into_inner().id;
        let measurements: Vec<Measurement> = self
            .workout_handler
            .get_measurements(workout_id)
            .await
            .unwrap();

        let (tx, rx) = channel(32);
        tokio::spawn(async move {
            for measurement in measurements.into_iter() {
                tx.send(Ok(measurement)).await.unwrap();
            }
            info!("Done sending");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn record_workout(
        &self,
        request: Request<Streaming<Measurement>>,
    ) -> GRPCResult<WorkoutSummary> {
        let mut stream = request.into_inner();

        let mut workout = Workout {
            km_ridden: 53.5,
            ..Default::default()
        };

        while let Some(measurement) = stream.next().await {
            info!("Measurement = {:?}", &measurement);
            workout.measurements.push(measurement?.clone());
        }
        info!("Recording done");

        let summary = self.workout_handler.save_workout(&workout).await;

        Ok(Response::new(summary))
    }

    async fn create_workout_plan(
        &self,
        request: Request<WorkoutPlan>,
    ) -> GRPCResult<WorkoutPlanToken> {
        let plan = request.into_inner();

        let workout_token = self.workout_handler.save_plan(&plan).await;

        Ok(Response::new(workout_token))
    }

    type RunWorkoutStream =
        Pin<Box<dyn Stream<Item = Result<ControlStep, Status>> + Send + 'static>>;

    async fn run_workout(
        &self,
        request: Request<Streaming<WorkoutStep>>,
    ) -> GRPCResult<Self::RunWorkoutStream> {
        let mut stream = request.into_inner();
        let mut training_steps: VecDeque<Step> = VecDeque::from(vec![]);

        let output = async_stream::try_stream! {
            while let Some(workout_step) = stream.next().await {
                let stype = StepType::try_from(workout_step?.stype).unwrap();
                match stype {
                    StepType::Starting => {
                        info!("Starting workout");
                        training_steps = VecDeque::from(vec![
                            Step { watts: 150, duration: 2},
                            Step { watts: 200, duration: 2},
                        ]);
                        yield next_control_step(stype, &mut training_steps);
                    },
                    StepType::InProgress => {
                        info!("Stepping over workout in progress");
                        yield next_control_step(stype, &mut training_steps);
                    },
                    StepType::Ending => {
                        info!("Ending workout");
                        yield end_workout();
                    },
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::RunWorkoutStream))
    }
}

fn next_control_step(
    stype: StepType,
    training_steps: &mut VecDeque<Step>,
) -> ControlStep {
    let step = training_steps.pop_front();
    let resistance = Some(step.unwrap_or_default().watts);
    info!("Next resistance: {:?}", resistance);

    ControlStep {
        stype: stype.into(),
        resistance,
        workout_summary_id: None,
    }
}

fn end_workout() -> ControlStep {
    ControlStep {
        stype: StepType::Ending.into(),
        resistance: None,
        workout_summary_id: Some(1),
    }
}

impl std::ops::Add for Measurement {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            speed: self.speed + other.speed,
            watts: self.watts + other.watts,
            rpm: self.rpm + other.rpm,
            heartrate: self.heartrate + other.heartrate,
            resistance: self.resistance + other.resistance,
        }
    }
}
