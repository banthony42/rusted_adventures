use common::database::model::location::UpdateLocationDestination;
use common::grpc_codegen::EntityDespawn;
use common::grpc_codegen::{
    server_entity_event::Event::{EntityDespawnEvent, EntityMoveEvent, EntitySpawnEvent},
    EntityMove, EntitySpawn,
};
use common::grpc_codegen::{
    Coord as RpcCoord, Entity as RpcEntity, Location as RpcLocation, ServerEntityEvent,
};

pub trait ServerEntityEventExtension {
    fn movement(uuid: String, location: RpcLocation) -> Self;
    fn spawn(entity: RpcEntity) -> Self;
    fn despawn(uuid: String) -> Self;
}

impl ServerEntityEventExtension for ServerEntityEvent {
    fn movement(uuid: String, location: RpcLocation) -> Self {
        ServerEntityEvent {
            event: Some(EntityMoveEvent(EntityMove {
                uuid: uuid,
                new_location: Some(location),
            })),
        }
    }

    fn spawn(entity: RpcEntity) -> Self {
        ServerEntityEvent {
            event: Some(EntitySpawnEvent(EntitySpawn {
                new_entity: Some(entity),
            })),
        }
    }

    fn despawn(uuid: String) -> Self {
        ServerEntityEvent {
            event: Some(EntityDespawnEvent(EntityDespawn { uuid })),
        }
    }
}

pub trait RpcLocationExtension {
    fn into_update_destination(&self) -> Option<UpdateLocationDestination>;
}

impl RpcLocationExtension for RpcLocation {
    fn into_update_destination(&self) -> Option<UpdateLocationDestination> {
        if let Some(map) = self.cell {
            return Some(UpdateLocationDestination {
                destination: Some(map.into()),
            });
        }
        None
    }
}

pub trait RpcCoordExtension {
    fn into_destination(&self) -> RpcLocation;
}

impl RpcCoordExtension for RpcCoord {
    fn into_destination(&self) -> RpcLocation {
        RpcLocation {
            map: None,
            cell: Some(self.clone()),
        }
    }
}
