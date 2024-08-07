use std::pin::Pin;

use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use crate::cycling_tracker::cycling_tracker_server::CyclingTracker;
use crate::cycling_tracker::{Measurement, Workout, WorkoutRequest, WorkoutSummary};
use crate::handler::{SessionHandler, WorkoutHandler};

type GRPCResult<T> = Result<Response<T>, Status>;

#[derive(Clone)]
pub struct CyclingTrackerService {
    workout_handler: WorkoutHandler,
    session_handler: SessionHandler,
}

impl CyclingTrackerService {
    pub fn new(
        workout_handler: WorkoutHandler,
        session_handler: SessionHandler,
    ) -> Self {
        Self {
            workout_handler,
            session_handler,
        }
    }
}

#[tonic::async_trait]
impl CyclingTracker for CyclingTrackerService {
    async fn save_workout(
        &self,
        request: Request<Workout>,
    ) -> GRPCResult<WorkoutSummary> {
        self.session_handler.verify_session_token(&request)?;

        let workout = request.into_inner();

        let summary = self.workout_handler.save_workout(&workout).await;

        Ok(Response::new(summary))
    }

    type GetMeasurementsStream = ReceiverStream<Result<Measurement, Status>>;

    async fn get_measurements(
        &self,
        request: Request<WorkoutRequest>,
    ) -> GRPCResult<Self::GetMeasurementsStream> {
        self.session_handler.verify_session_token(&request)?;

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
        self.session_handler.verify_session_token(&request)?;

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

    type GetCurrentAveragesStream =
        Pin<Box<dyn Stream<Item = Result<WorkoutSummary, Status>> + Send + 'static>>;

    async fn get_current_averages(
        &self,
        request: Request<Streaming<Measurement>>,
    ) -> GRPCResult<Self::GetCurrentAveragesStream> {
        self.session_handler.verify_session_token(&request)?;

        let mut stream = request.into_inner();

        let mut workout = Workout::default();
        let workout_handler = self.workout_handler.clone();

        let output = async_stream::try_stream! {
            while let Some(measurement) = stream.next().await {
                workout.measurements.push(measurement.clone()?);
                workout.km_ridden += measurement?.speed;
                yield workout_handler.create_summary(&workout);
            }
        };

        Ok(Response::new(
            Box::pin(output) as Self::GetCurrentAveragesStream
        ))
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
        }
    }
}
