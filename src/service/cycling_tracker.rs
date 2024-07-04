use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

use crate::cycling_tracker::cycling_tracker_server::CyclingTracker;
use crate::cycling_tracker::{
    Activity, ActivityRequest, ActivityStep, ActivitySummary, ControlStep, Measurement,
    TrainingPlan, TrainingPlanToken,
};
use crate::GRPCResult;

pub struct CyclingTrackerService {}

#[tonic::async_trait]
impl CyclingTracker for CyclingTrackerService {
    async fn save_activity(
        &self,
        request: Request<Activity>,
    ) -> GRPCResult<ActivitySummary> {
        Ok(Response::new(ActivitySummary::default()))
    }

    type GetMeasurementsStream = ReceiverStream<Result<Measurement, Status>>;

    async fn get_measurements(
        &self,
        request: Request<ActivityRequest>,
    ) -> GRPCResult<Self::GetMeasurementsStream> {
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            tx.send(Ok(Measurement::default())).await.unwrap();

            println!(" /// done sending");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn record_activity(
        &self,
        request: Request<Streaming<Measurement>>,
    ) -> GRPCResult<ActivitySummary> {
        let mut stream = request.into_inner();

        let mut summary = ActivitySummary::default();

        while let Some(measurement) = stream.next().await {
            let measurement = measurement?;

            println!("Measurement = {:?}", measurement);
        }

        Ok(Response::new(summary))
    }

    async fn create_training_plan(
        &self,
        request: Request<TrainingPlan>,
    ) -> GRPCResult<TrainingPlanToken> {
        Ok(Response::new(TrainingPlanToken::default()))
    }

    type ExecuteTrainingPlanStream =
        Pin<Box<dyn Stream<Item = Result<ControlStep, Status>> + Send + 'static>>;

    async fn execute_training_plan(
        &self,
        request: Request<Streaming<ActivityStep>>,
    ) -> GRPCResult<Self::ExecuteTrainingPlanStream> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(activity_step) = stream.next().await {
                yield ControlStep::default();
            }
        };

        Ok(Response::new(
            Box::pin(output) as Self::ExecuteTrainingPlanStream
        ))
    }
}
