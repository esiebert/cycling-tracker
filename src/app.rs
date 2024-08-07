use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use thiserror::Error;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_reflection::server::Builder as ReflectionServerBuilder;

use crate::cycling_tracker;
use crate::grpc::{
    auth::SessionAuthService, cycling_tracker::CyclingTrackerService,
    BuildError as GRPCBuildError, Builder as GRPCBuilder, GRPC,
};
use crate::handler::{
    RedisHandler, SQLiteHandler, SessionHandler, UserHandler, WorkoutHandler,
};
use crate::FILE_DESCRIPTOR_SET;

pub struct App {
    grpc: GRPC,
}

impl App {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub async fn run(mut self) -> Result<()> {
        tokio::select! {
            e = self.grpc.run() => {
                e
            }
        }
    }

    pub async fn run_tcp(mut self, tcp_listener: TcpListenerStream) -> Result<()> {
        tokio::select! {
            e = self.grpc.run_tcp(tcp_listener) => {
                e
            }
        }
    }
}

pub struct Builder {
    grpc: Option<GRPC>,
    db: Option<SqlitePool>,
    redis: Option<redis::Client>,
}

impl Builder {
    fn new() -> Self {
        Self {
            grpc: None,
            db: None,
            redis: None,
        }
    }

    pub async fn setup_database(mut self, db_url: &str) -> Result<Self, BuildError> {
        Sqlite::create_database(db_url)
            .await
            .map_err(|e| BuildError::DbCreationFailed(format!("{e:?}")))?;

        let db = SqlitePool::connect(db_url)
            .await
            .map_err(|e| BuildError::DbConnectionFailed(format!("{e:?}")))?;

        sqlx::migrate!()
            .run(&db)
            .await
            .map_err(|e| BuildError::DbMigrationFailed(format!("{e:?}")))?;

        self.db = Some(db);

        Ok(self)
    }

    pub fn with_db(mut self, db: SqlitePool) -> Self {
        self.db = Some(db);
        self
    }

    pub fn setup_redis(mut self, redis_url: &str) -> Result<Self, BuildError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| BuildError::RedisFailed(format!("{e:?}")))?;

        // Test if a connection can be opened
        let _ = client
            .get_connection()
            .map_err(|e| BuildError::RedisFailed(format!("{e:?}")))?;

        self.redis = Some(client);

        Ok(self)
    }

    pub fn with_redis(mut self, redis: redis::Client) -> Self {
        self.redis = Some(redis);
        self
    }

    pub async fn setup_grpc(
        mut self,
        host_url: &str,
        with_tls: bool,
    ) -> Result<Self, BuildError> {
        self.db.as_ref().ok_or(BuildError::DatabaseNotSet)?;
        let sqlite_handler = SQLiteHandler {
            db: self.db.clone().unwrap(),
        };

        self.redis.as_ref().ok_or(BuildError::RedisNotSet)?;
        let redis_handler = RedisHandler {
            client: self.redis.clone().unwrap(),
        };

        let cts =
            cycling_tracker::CyclingTrackerServer::new(CyclingTrackerService::new(
                WorkoutHandler {
                    sqlite_handler: sqlite_handler.clone(),
                },
                SessionHandler {
                    redis_handler: redis_handler.clone(),
                },
            ));

        let refl = ReflectionServerBuilder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build()
            .map_err(|e| BuildError::ReflectionBuildError(e.to_string()))?;

        let mut grpc_builder = GRPCBuilder::new().with_addr(host_url)?;

        if with_tls {
            grpc_builder = grpc_builder.with_tls()?;
        }

        let auth = cycling_tracker::SessionAuthServer::new(SessionAuthService::new(
            UserHandler { sqlite_handler },
            SessionHandler { redis_handler },
        ));
        let grpc = grpc_builder
            .add_auth_service(auth)
            .add_reflection_service(refl)
            .add_ct_service(cts)
            .build()?;

        self.grpc = Some(grpc);
        Ok(self)
    }

    pub fn build(self) -> Result<App, BuildError> {
        let grpc = self.grpc.ok_or(BuildError::GRPCNotSet)?;

        Ok(App { grpc })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Redis not set: required to setup gRPC")]
    RedisNotSet,
    #[error("Failed to connect to redis: {0}")]
    RedisFailed(String),
    #[error("Database not set: required to setup gRPC")]
    DatabaseNotSet,
    #[error("Failed to create database: {0}")]
    DbCreationFailed(String),
    #[error("Failed to connect to database: {0}")]
    DbConnectionFailed(String),
    #[error("Failed to migrate database: {0}")]
    DbMigrationFailed(String),
    #[error("Failed to build gRPC: {0}")]
    GRPCBuildFailure(#[from] GRPCBuildError),
    #[error("Failed to build reflection server: {0}")]
    ReflectionBuildError(String),
    #[error("gRPC service not set")]
    GRPCNotSet,
}
