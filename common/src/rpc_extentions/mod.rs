use diesel_geometry::data_types::PgPoint;

use crate::database::model::location::{UpdateLocation, UpdateLocationDestination};
use crate::grpc_codegen::EntityDespawn;
use crate::grpc_codegen::{
    server_entity_event::Event::{EntityDespawnEvent, EntityMoveEvent, EntitySpawnEvent},
    EntityMove, EntitySpawn,
};
use crate::grpc_codegen::{
    Coord as RpcCoord, Entity as RpcEntity, Location as RpcLocation, ServerEntityEvent,
};
use crate::{CellCoord, MapCoord};
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
    fn into_update_location(&self) -> Option<UpdateLocation>;
    fn into_update_destination(&self) -> Option<UpdateLocationDestination>;
    fn into_cell_map(&self) -> Option<(RpcCoord, RpcCoord)>;
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

    // For now i don't find the tonic / gRPC syntax or trick to force a field to not be a rust Option
    // entity.proto Location.map and Location.cell should be always defined
    fn into_cell_map(&self) -> Option<(RpcCoord, RpcCoord)> {
        Some((self.cell?, self.map?))
    }

    fn into_update_location(&self) -> Option<UpdateLocation> {
        let (cell, map) = self.into_cell_map()?;
        Some(UpdateLocation {
            map: map.into_pg_point(),
            cell: cell.into_pg_point(),
        })
    }
}

pub trait RpcCoordExtension {
    fn into_destination(self) -> RpcLocation;
    fn into_cell(self) -> CellCoord;
    fn into_map(self) -> MapCoord;
    fn into_pg_point(self) -> PgPoint;
}

impl RpcCoordExtension for RpcCoord {
    fn into_destination(self) -> RpcLocation {
        RpcLocation {
            map: None,
            cell: Some(self),
        }
    }

    fn into_cell(self) -> CellCoord {
        CellCoord {
            x: self.x,
            y: self.y,
        }
    }

    fn into_map(self) -> MapCoord {
        MapCoord {
            x: self.x as i8, // protobuf smallest int type is i32
            y: self.y as i8,
        }
    }

    fn into_pg_point(self) -> PgPoint {
        PgPoint(self.x as f64, self.y as f64)
    }
}
