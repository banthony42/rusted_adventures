use std::thread;

use ::common::grpc_codegen::rpg_chat_server::RpgChatServer;
use common::{
    authenticator::Authenticator, database::db::Database,
    grpc_codegen::rpg_entity_server::RpgEntityServer,
};
use services::chat::RpgChatService;
use services::entities::RpgEntityService;
use tokio::sync::mpsc::Receiver;
use tonic::transport::Server;

use common::grpc_codegen::rpg_authenticate_server::RpgAuthenticateServer;

use services::authenticate::RpgAuthenticateService;
use tonic::{Request, Status};

use tracing_subscriber::EnvFilter;
use world::engine::{WorldEngine, WorldEvent};

pub mod proto {
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../common/grpc_codegen/rpg_services_descriptor.bin");
}

mod generics;
mod services;
mod world;

fn run_world_engine_on_another_thread() -> Receiver<WorldEvent> {
    let (mut world_engine, world_rx) = WorldEngine::new();
    thread::spawn(move || {
        world_engine.run();
    });

    return world_rx;
}

fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let Some(login_data) = req.metadata().get("login") else {
        return Err(Status::invalid_argument("login or authorization not found"));
    };

    let Some(token_data) = req.metadata().get("authorization") else {
        return Err(Status::invalid_argument("login or authorization not found"));
    };

    let Ok(login) = login_data.to_str() else {
        return Err(Status::invalid_argument("Invalid login or authorization"));
    };

    let Ok(token) = token_data.to_str() else {
        return Err(Status::invalid_argument("Invalid login or authorization"));
    };

    let mut user = Authenticator::new(login);

    // Ensure the account is not lockout because of any bruteforce
    // https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html#account-lockout
    user.is_allowed()?;

    // Finally ensure it's connected with a valid session
    user.is_connected(token)?;

    Ok(req)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Run diesel migration
    Database::new()
        .run_migration()
        .expect("Server: Fail to run database migration");

    // For setup simplicity the WorldEngine coexist within the Grpc Server.
    // I insist on the term 'WorldENGINE' because it's not a server since it not handle connections or requests.
    // We spawn a new thread where the WorldEngine will run.
    // Goal is to share a Mutex on the Database access to ensure DB READ/WRITE atomicity
    // Then the WorldEngine should update the world over the time (READ/WRITE)
    // and the Grpc server will just answer to client request READING the DB when needed.
    // The only case where the Grpc Server will WRITE to the DB is to store the session token.
    // Therefore with this approach it means that the Game data are stored in the postgresql DB.
    // I know it's not the best solution, maybe a redis should be better for performance
    // But again, the goal of the project is to learn rust, and i want to avoid
    // heavy setup configuration / prerequisite.
    // In addition i already know redis, where i totally discover and learn SQL like DB.
    let world_rx = run_world_engine_on_another_thread();

    // Run tonic gRPC server
    let addr = "0.0.0.0:2121".parse()?;
    let rpg_authenticate = RpgAuthenticateService::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("Server: Fail to build tonic server reflection.");

    tracing::info!("Starting server {addr}");

    Server::builder()
        .trace_fn(|request| {
            tracing::info_span!(
                "Request ",
                method = %request.method(),
                uri = %request.uri().path()
            )
        })
        .add_service(reflection_service)
        .add_service(RpgAuthenticateServer::new(rpg_authenticate))
        .add_service(RpgChatServer::with_interceptor(
            RpgChatService::new(),
            auth_interceptor,
        ))
        .add_service(RpgEntityServer::with_interceptor(
            RpgEntityService::new(world_rx),
            auth_interceptor,
        ))
        .serve(addr)
        .await?;

    Ok(())
}

/*
    Long lived Full duplex streaming (bidirectional streaming RPC) issues:
    At least one side is reading to avoid deadlock:
        OK: since both client and server are always reading the streams
    Load balancing (memory + cpu):
        KO: need more informations to understand what is needed
            I assume memory issue are channel with data in it waiting to be received
            and CPU issue would be massive data to process from channel.
            Not enough CPU mean channel memory growth
    Application should have a retry mecanism to reconnect:
        KO: Should be implemented on the client side since it's the client who initiate the connection
            On the server we should detect any disconnection and free data related to the disconnected client
*/
