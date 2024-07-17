use crate::api::Message;
use crate::cycling_tracker::cycling_tracker_server::CyclingTracker;
use crate::cycling_tracker::{
    workout_plan::Step, ControlStep, Measurement, StepType, Workout, WorkoutPlan,
    WorkoutPlanToken, WorkoutRequest, WorkoutStep, WorkoutSummary,
};
use std::collections::VecDeque;
use std::pin::Pin;
use tokio::sync::mpsc::{channel, Sender};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

type GRPCResult<T> = Result<Response<T>, Status>;

#[derive(Clone)]
pub struct CyclingTrackerService {
    sqlite: Sender<Message>,
}

impl CyclingTrackerService {
    pub fn new(sqlite: Sender<Message>) -> Self {
        Self { sqlite: sqlite }
    }
}

#[tonic::async_trait]
impl CyclingTracker for CyclingTrackerService {
    async fn save_workout(
        &self,
        request: Request<Workout>,
    ) -> GRPCResult<WorkoutSummary> {
        let workout = request.into_inner();

        let readings = workout.measurements.len();

        if readings == 0 {
            return Ok(Response::new(WorkoutSummary {
                id: 1,
                km_ridden: workout.km_ridden,
                ..Default::default()
            }));
        }

        let acc_measurements = workout
            .measurements
            .clone()
            .into_iter()
            .reduce(|acc, e| acc + e)
            .unwrap();

        let workout_summary = WorkoutSummary {
            id: 1,
            km_ridden: workout.km_ridden,
            avg_speed: acc_measurements.speed / readings as f32,
            avg_watts: acc_measurements.watts / readings as i32,
            avg_rpm: acc_measurements.rpm / readings as i32,
            avg_heartrate: acc_measurements.heartrate / readings as i32,
            measurements: workout.measurements,
        };

        let _ = self
            .sqlite
            .send(Message::SaveWorkout("Workout saved".to_string()))
            .await;

        Ok(Response::new(workout_summary))
    }

    type GetMeasurementsStream = ReceiverStream<Result<Measurement, Status>>;

    async fn get_measurements(
        &self,
        _request: Request<WorkoutRequest>,
    ) -> GRPCResult<Self::GetMeasurementsStream> {
        let workout_summaries = vec![WorkoutSummary {
            id: 1,
            km_ridden: 10.0,
            avg_speed: 30.0,
            avg_watts: 300,
            avg_rpm: 95,
            avg_heartrate: 180,
            measurements: vec![
                Measurement {
                    speed: 29.0,
                    watts: 300,
                    rpm: 95,
                    resistance: 700,
                    heartrate: 180,
                },
                Measurement {
                    speed: 30.0,
                    watts: 300,
                    rpm: 95,
                    resistance: 700,
                    heartrate: 180,
                },
                Measurement {
                    speed: 31.0,
                    watts: 300,
                    rpm: 95,
                    resistance: 700,
                    heartrate: 180,
                },
            ],
        }];
        let (tx, rx) = channel(32);

        tokio::spawn(async move {
            for measurement in workout_summaries
                .get(0)
                .unwrap()
                .measurements
                .clone()
                .into_iter()
            {
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

        let mut summary = WorkoutSummary::default();

        let mut acc_measurements = Measurement::default();

        while let Some(measurement) = stream.next().await {
            info!("Measurement = {:?}", &measurement);
            let measurement = measurement?;
            summary.measurements.push(measurement.clone());
            summary.km_ridden += 1.5;
            acc_measurements = acc_measurements + measurement;
        }
        info!("Recording done");

        let readings = summary.measurements.len();
        summary.avg_speed = acc_measurements.speed / readings as f32;
        summary.avg_watts = acc_measurements.watts / readings as i32;
        summary.avg_rpm = acc_measurements.rpm / readings as i32;
        summary.avg_heartrate = acc_measurements.heartrate / readings as i32;

        Ok(Response::new(summary))
    }

    async fn create_workout_plan(
        &self,
        _request: Request<WorkoutPlan>,
    ) -> GRPCResult<WorkoutPlanToken> {
        Ok(Response::new(WorkoutPlanToken::default()))
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
                        let training_plan = WorkoutPlan {
                            steps: vec![
                                Step { watts: 150, duration: 2},
                                Step { watts: 200, duration: 2},
                            ]
                        };
                        training_steps = VecDeque::from(training_plan.steps);
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
        resistance: resistance,
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
