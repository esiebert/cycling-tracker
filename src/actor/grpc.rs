use tonic_reflection;

use crate::useless_box::route_guide_server::RouteGuideServer;
use crate::useless_box::session_auth_server::SessionAuthServer;
use crate::useless_box::FILE_DESCRIPTOR_SET;
use tonic::{
    metadata::MetadataValue,
    transport::{Identity, Server, ServerTlsConfig},
    Request, Status,
};

use crate::service::RouteGuideService;
use crate::service::SessionAuthService;
use crate::util::data::populate;

pub struct GRPCActor {
    grpc_host_url: String,
}

impl GRPCActor {
    pub fn new(grpc_host_url: String) -> Self {
        Self {
            grpc_host_url: grpc_host_url,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data_dir = std::path::PathBuf::from_iter([std::env!("CARGO_MANIFEST_DIR"), "data/tls"]);
        let cert = std::fs::read_to_string(data_dir.join("server.pem"))?;
        let key = std::fs::read_to_string(data_dir.join("server.key"))?;
        let identity = Identity::from_pem(cert, key);

        let reflection_svc = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()
            .unwrap();

        let route_svc = RouteGuideServer::with_interceptor(
            RouteGuideService {
                features: populate(),
            },
            check_session_token,
        );
        let auth_svc = SessionAuthServer::new(SessionAuthService {});

        let addr = self.grpc_host_url.parse().unwrap();
        println!("RouteGuideServer listening on: {}", addr);

        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(reflection_svc)
            .add_service(route_svc)
            .add_service(auth_svc)
            .serve(addr)
            .await?;

        Ok(())
    }
}

fn check_session_token(req: Request<()>) -> Result<Request<()>, Status> {
    let token: MetadataValue<_> = "Bearer session-token".parse().unwrap();

    match req.metadata().get("Authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("Invalid session token")),
    }
}
