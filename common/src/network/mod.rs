use crate::grpc_codegen::EntityDespawn;
use crate::grpc_codegen::{
    server_entity_event::Event::{Despawn, Move, Spawn},
    EntityMove, EntitySpawn,
};
use crate::grpc_codegen::{Entity as RpcEntity, Location as RpcLocation, ServerEntityEvent};

pub mod conversion;

pub trait ServerEntityEventExtension {
    fn movement(uuid: String, location: RpcLocation) -> Self;
    fn spawn(entity: RpcEntity) -> Self;
    fn despawn(uuid: String) -> Self;
}

impl ServerEntityEventExtension for ServerEntityEvent {
    fn movement(uuid: String, location: RpcLocation) -> Self {
        ServerEntityEvent {
            event: Some(Move(EntityMove {
                uuid,
                new_location: Some(location),
            })),
        }
    }

    fn spawn(entity: RpcEntity) -> Self {
        ServerEntityEvent {
            event: Some(Spawn(EntitySpawn {
                new_entity: Some(entity),
            })),
        }
    }

    fn despawn(uuid: String) -> Self {
        ServerEntityEvent {
            event: Some(Despawn(EntityDespawn { uuid })),
        }
    }
}
