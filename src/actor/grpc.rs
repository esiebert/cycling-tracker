use tonic_reflection::server::Builder;

use crate::cycling_tracker::cycling_tracker_server::CyclingTrackerServer;
use crate::cycling_tracker::session_auth_server::SessionAuthServer;
use crate::cycling_tracker::FILE_DESCRIPTOR_SET;
use tonic::{
    metadata::MetadataValue,
    transport::{Identity, Server, ServerTlsConfig},
    Request, Status,
};

use crate::service::CyclingTrackerService;
use crate::service::SessionAuthService;

use tracing::info;

pub struct GRPCActor {
    grpc_host_url: String,
    cyclink_tracker_service: CyclingTrackerService,
}

impl GRPCActor {
    pub fn new(
        grpc_host_url: String,
        cyclink_tracker_service: CyclingTrackerService,
    ) -> Self {
        Self {
            grpc_host_url: grpc_host_url,
            cyclink_tracker_service: cyclink_tracker_service,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let reflection_svc = Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()
            .unwrap();

        let ct_svc = CyclingTrackerServer::with_interceptor(
            self.cyclink_tracker_service.clone(),
            check_session_token,
        );

        let auth_svc = SessionAuthServer::new(SessionAuthService {});

        let addr = self.grpc_host_url.parse().unwrap();
        info!("CyclingTracker listening on: {}", addr);

        Server::builder()
            .tls_config(config_tls()?)?
            .add_service(reflection_svc)
            .add_service(ct_svc)
            .add_service(auth_svc)
            .serve(addr)
            .await?;

        Ok(())
    }
}

fn config_tls() -> Result<ServerTlsConfig, Box<dyn std::error::Error>> {
    let data_dir =
        std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data/tls"]);
    let cert = std::fs::read_to_string(data_dir.join("server.pem"))
        .map_err(|err| format!("Error reading public key file: {}", err))?;
    let key = std::fs::read_to_string(data_dir.join("server.key"))
        .map_err(|err| format!("Error reading private key file: {}", err))?;
    Ok(ServerTlsConfig::new().identity(Identity::from_pem(cert, key)))
}

fn check_session_token(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer session-token".parse().unwrap();

    match req.metadata().get("Authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid session token")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_check_session_token_valid() {
        let mut req = Request::new(());
        let token: MetadataValue<_> = "Bearer session-token".parse().unwrap();
        req.metadata_mut().insert("Authorization", token);

        let result = check_session_token(req);

        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_check_session_token_invalid() {
        let mut req = Request::new(());
        let token: MetadataValue<_> = "Bearer invalid-token".parse().unwrap();
        req.metadata_mut().insert("Authorization", token);

        let result = check_session_token(req);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn test_check_session_token_no_token() {
        let req = Request::new(());

        let result = check_session_token(req);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }
}
