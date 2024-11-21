use tokio::runtime::{Builder, Runtime};
use tonic::transport::Server;

use services::authenticate::grpc_codegen::rpg_authenticate_server::RpgAuthenticateServer;
use services::authenticate::RpgAuthenticateService;
use world::engine::WorldEngine;

pub mod proto {
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../common/GRPC_codegen/rpg_services_descriptor.bin");
}

mod services;
mod world;

fn run_world_engine_on_another_thread() -> Runtime {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("RPG World Engine")
        .enable_all()
        .build()
        .unwrap();

    let world_engine = WorldEngine::new();
    runtime.spawn(async move {
        world_engine.run().await;
    });

    return runtime;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:2121".parse()?;
    let rpg_authenticate = RpgAuthenticateService::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();
    // For setup simplicity the WorldEngine coexist within the Grpc Server.
    // I insist on the term 'WorldENGINE' because it's not a server since it not handle connections or requests.
    // We spawn a new thread where the WorldEngine will run (TODO: maybe we don't need the WorldEngine to be async for now)
    // Goal is to share a Mutex on the Database access to ensure DB read/write atomicity
    // Then the WorldEngine should update the world over the time (read/write)
    // and the Grpc server will just answer to client request reading the DB when needed.
    // Therefore with this approach it means that the Game States are stored in the postgresql DB.
    // I know it's not the best solution, maybe a redis should be better for performance
    // But again, the goal of the project is to learn rust, and i want to avoid
    // heavy setup configuration / prerequisite.
    // In addition i already know redis, where i totally discover and learn SQL like DB.
    let _rt = run_world_engine_on_another_thread();

    Server::builder()
        .add_service(reflection_service)
        .add_service(RpgAuthenticateServer::new(rpg_authenticate))
        .serve(addr)
        .await?;

    Ok(())
}
