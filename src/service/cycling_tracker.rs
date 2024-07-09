use crate::cycling_tracker::cycling_tracker_server::CyclingTracker;
use crate::cycling_tracker::{
    training_plan::Step, Activity, ActivityRequest, ActivityStep, ActivitySummary,
    ControlStep, Measurement, StepType, TrainingPlan, TrainingPlanToken,
};
use crate::GRPCResult;
use std::collections::VecDeque;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;
pub struct CyclingTrackerService {}

#[tonic::async_trait]
impl CyclingTracker for CyclingTrackerService {
    async fn save_activity(
        &self,
        request: Request<Activity>,
    ) -> GRPCResult<ActivitySummary> {
        let activity = request.into_inner();

        let readings = activity.measurements.len();

        if readings == 0 {
            return Ok(Response::new(ActivitySummary {
                id: 1,
                km_ridden: activity.km_ridden,
                ..Default::default()
            }));
        }

        let acc_measurements = activity
            .measurements
            .clone()
            .into_iter()
            .reduce(|acc, e| acc + e)
            .unwrap();

        let activity_summary = ActivitySummary {
            id: 1,
            km_ridden: activity.km_ridden,
            avg_speed: acc_measurements.speed / readings as f32,
            avg_watts: acc_measurements.watts / readings as i32,
            avg_rpm: acc_measurements.rpm / readings as i32,
            avg_heartrate: acc_measurements.heartrate / readings as i32,
            measurements: activity.measurements,
        };

        Ok(Response::new(activity_summary))
    }

    type GetMeasurementsStream = ReceiverStream<Result<Measurement, Status>>;

    async fn get_measurements(
        &self,
        _request: Request<ActivityRequest>,
    ) -> GRPCResult<Self::GetMeasurementsStream> {
        let activity_summaries = vec![ActivitySummary {
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
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for measurement in activity_summaries
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

    async fn record_activity(
        &self,
        request: Request<Streaming<Measurement>>,
    ) -> GRPCResult<ActivitySummary> {
        let mut stream = request.into_inner();

        let mut summary = ActivitySummary::default();

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

    async fn create_training_plan(
        &self,
        _request: Request<TrainingPlan>,
    ) -> GRPCResult<TrainingPlanToken> {
        Ok(Response::new(TrainingPlanToken::default()))
    }

    type ExecuteTrainingPlanStream =
        Pin<Box<dyn Stream<Item = Result<ControlStep, Status>> + Send + 'static>>;

    async fn execute_training_plan(
        &self,
        request: Request<Streaming<ActivityStep>>,
    ) -> GRPCResult<Self::ExecuteTrainingPlanStream> {
        info!("Starting training plan");
        let mut stream = request.into_inner();

        let mut training_steps: VecDeque<Step> = VecDeque::from(vec![]);

        let output = async_stream::try_stream! {
            while let Some(activity_step) = stream.next().await {
                let stype = StepType::try_from(activity_step?.stype).unwrap();
                match stype {
                    StepType::Starting => {
                        let training_plan = TrainingPlan {
                            steps: vec![
                                Step { watts: 150, duration: 2},
                                Step { watts: 200, duration: 2},
                            ]
                        };
                        training_steps = VecDeque::from(training_plan.steps);
                        yield next_control_step(stype, &mut training_steps);
                    },
                    StepType::InProgress => {
                        yield next_control_step(stype, &mut training_steps);
                    },
                    StepType::Ending => {
                        yield end_workout();
                    },
                }
            }
        };

        Ok(Response::new(
            Box::pin(output) as Self::ExecuteTrainingPlanStream
        ))
    }
}

fn next_control_step(
    stype: StepType,
    training_steps: &mut VecDeque<Step>,
) -> ControlStep {
    let step = training_steps.pop_front();
    let resistance = Some(step.unwrap_or_default().watts);

    ControlStep {
        stype: stype.into(),
        resistance: resistance,
        activity_summary_id: None,
    }
}

fn end_workout() -> ControlStep {
    ControlStep {
        stype: StepType::Ending.into(),
        resistance: None,
        activity_summary_id: Some(1),
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
