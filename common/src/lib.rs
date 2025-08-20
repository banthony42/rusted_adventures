pub mod authenticator;
pub mod character;
pub mod database;
pub mod record;
pub mod utils;

pub mod grpc_codegen {
    include!("../grpc_codegen/rpg.package.rs");
}
