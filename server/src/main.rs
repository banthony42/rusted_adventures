use ::common::grpc_codegen::rpg_chat_server::RpgChatServer;
use common::{authenticator::Authenticator, grpc_codegen::rpg_entity_server::RpgEntityServer};
use services::chat::RpgChatService;
use services::entities::RpgEntityService;
use tokio::runtime::{Builder, Runtime};
use tonic::{metadata::MetadataValue, transport::Server};

use common::grpc_codegen::rpg_authenticate_server::RpgAuthenticateServer;

use services::authenticate::RpgAuthenticateService;
use tonic::{Request, Status};
use world::engine::WorldEngine;

pub mod proto {
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../common/grpc_codegen/rpg_services_descriptor.bin");
}

mod generics;
mod services;
mod world;

fn run_world_engine_on_another_thread() -> Runtime {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("RPG World Engine")
        .enable_all()
        .build()
        .unwrap();

    let mut world_engine = WorldEngine::new();
    runtime.spawn(async move {
        world_engine.run();
    });

    return runtime;
}

fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("login") {
        Some(login_md) => {
            let login = login_md
                .to_str()
                .map_err(|e| Status::unauthenticated(format!("Error getting login: {:?}", e)))?;

            let mut user = Authenticator::new(login);
            let token: MetadataValue<_> = user
                .get_token()
                .ok_or(Status::unauthenticated("user not authenticated"))?
                .parse()
                .map_err(|e| Status::unauthenticated(format!("{:?}", e)))?;

            match req.metadata().get("authorization") {
                Some(t) if token == t => Ok(req),
                _ => Err(Status::unauthenticated("No valid auth token")),
            }
        }
        _ => Err(Status::unauthenticated("login not found")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:21210".parse()?;
    let rpg_authenticate = RpgAuthenticateService::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

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
    let _rt = run_world_engine_on_another_thread();

    Server::builder()
        .add_service(reflection_service)
        .add_service(RpgAuthenticateServer::new(rpg_authenticate))
        .add_service(RpgChatServer::with_interceptor(
            RpgChatService::new(),
            auth_interceptor,
        ))
        .add_service(RpgEntityServer::with_interceptor(
            RpgEntityService::new(),
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
