pub mod authenticator;
pub mod database;
pub mod utils;

pub mod grpc_codegen {
    include!("../GRPC_codegen/rpg.package.rs");
}
