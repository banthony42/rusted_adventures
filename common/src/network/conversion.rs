use diesel_geometry::data_types::PgPoint;

use crate::{
    database::model::location::UpdateLocation,
    entities::{
        model::EntityModel,
        monsters::{Monster, MonsterType},
        player::{Player, PlayerType},
    },
    grpc_codegen::{entity::Family, Classes, Coord, Entity, Location, Species},
    CellCoord, MapCoord, Position,
};

#[derive(Debug, thiserror::Error)]
pub enum NetEntityError {
    #[error("Net Family is missing, can't build Player.")]
    MissingFamily,

    #[error("Species {0:?} from Net, can't be transform into Player.")]
    UnexpectedSpeciesAsPlayer(i32),

    #[error("Class {0:?} from Net, can't be transform into Monster.")]
    UnexpectedClassAsMonster(i32),

    #[error("Net Family::Class variant: {0} can't be transform into Player.")]
    UnexpectedClass(i32),

    #[error("Net Family::Species variant: {0} not found for Species")]
    SpeciesNotFound(i32),

    #[error("Net Family::Class variant: {0} not found for Species")]
    ClassNotFound(i32),

    #[error("Missing map coordinates from net, can't build Position.")]
    MissingMap,

    #[error("Missing cell coordinates from net, can't build Position.")]
    MissingCell,

    #[error("Missing Location from net, can't build Position.")]
    MissingLocation,

    #[error("Map coordinates are out of range, can't build Position.")]
    InvalidMapCoordinates,
}

impl From<CellCoord> for Coord {
    fn from(value: CellCoord) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Coord> for CellCoord {
    fn from(value: Coord) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<MapCoord> for Coord {
    fn from(value: MapCoord) -> Self {
        Self {
            x: value.x as i64,
            y: value.y as i64,
        }
    }
}

impl TryFrom<Coord> for MapCoord {
    type Error = NetEntityError;

    fn try_from(value: Coord) -> Result<Self, Self::Error> {
        Ok(Self {
            x: i8::try_from(value.x).map_err(|_| NetEntityError::InvalidMapCoordinates)?,
            y: i8::try_from(value.y).map_err(|_| NetEntityError::InvalidMapCoordinates)?,
        })
    }
}

impl From<Position> for Location {
    fn from(value: Position) -> Self {
        Self {
            cell: Some(value.cell.into()),
            map: Some(value.map.into()),
        }
    }
}

impl TryFrom<Location> for Position {
    type Error = NetEntityError;

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        let map = value.map.ok_or(NetEntityError::MissingMap)?;
        let cell = value.cell.ok_or(NetEntityError::MissingCell)?;

        Ok(Self {
            map: MapCoord::try_from(map)?,
            cell: CellCoord::from(cell),
        })
    }
}

impl TryFrom<Option<Location>> for Position {
    type Error = NetEntityError;

    fn try_from(value: Option<Location>) -> Result<Self, Self::Error> {
        match value {
            Some(location) => Position::try_from(location),
            None => Err(NetEntityError::MissingLocation),
        }
    }
}

impl From<PgPoint> for MapCoord {
    fn from(value: PgPoint) -> Self {
        MapCoord {
            x: value.0 as i8,
            y: value.1 as i8,
        }
    }
}

impl From<PgPoint> for Coord {
    fn from(val: PgPoint) -> Self {
        Coord {
            x: val.0 as i64,
            y: val.1 as i64,
        }
    }
}

impl From<Coord> for PgPoint {
    fn from(value: Coord) -> Self {
        PgPoint(value.x as f64, value.y as f64)
    }
}

impl TryFrom<Location> for UpdateLocation {
    type Error = NetEntityError;

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        let map = value.map.ok_or(NetEntityError::MissingMap)?;
        let cell = value.cell.ok_or(NetEntityError::MissingCell)?;

        Ok(UpdateLocation {
            map: map.into(),
            cell: cell.into(),
        })
    }
}

impl TryFrom<Option<Location>> for UpdateLocation {
    type Error = NetEntityError;

    fn try_from(value: Option<Location>) -> Result<Self, Self::Error> {
        match value {
            Some(location) => UpdateLocation::try_from(location),
            None => Err(NetEntityError::MissingLocation),
        }
    }
}

impl TryFrom<Entity> for PlayerType {
    type Error = NetEntityError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        match value.family {
            None => Err(NetEntityError::MissingFamily),
            val if val == Some(Family::Class(Classes::Warrior as i32)) => {
                Ok(Self::Warrior(Player::new(value.try_into()?)))
            }
            val if val == Some(Family::Class(Classes::Mage as i32)) => {
                Ok(Self::Mage(Player::new(value.try_into()?)))
            }
            Some(Family::Class(c)) => Err(NetEntityError::ClassNotFound(c)),
            Some(Family::Species(s)) => Err(NetEntityError::UnexpectedSpeciesAsPlayer(s)),
        }
    }
}

impl TryFrom<Entity> for MonsterType {
    type Error = NetEntityError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        match value.family {
            None => Err(NetEntityError::MissingFamily),
            val if val == Some(Family::Species(Species::Bouftou as i32)) => {
                Ok(Self::Bouftou(Monster::new(value.try_into()?)))
            }
            val if val == Some(Family::Species(Species::Crabedoeuf as i32)) => {
                Ok(Self::Crabedoeuf(Monster::new(value.try_into()?)))
            }
            Some(Family::Species(s)) => Err(NetEntityError::SpeciesNotFound(s)),
            Some(Family::Class(c)) => Err(NetEntityError::UnexpectedClassAsMonster(c)),
        }
    }
}

impl TryFrom<Entity> for EntityModel {
    type Error = NetEntityError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.name,
            value.uuid,
            value.location.try_into()?,
        ))
    }
}

// To write all conversion here
// We need common model of world objects (player, entity, monster)
// these objects will be used by server (instanciate them in the world to simulate them)
// and by client to have data to represent
// to do that we need to have all this object at the same place
// leading me to move client/src/entities/model.rs 90% code here
// but all Animations / Frame stuff are only for the client
// leading me to rework IEntity trait
// splitting it in two maybe
// one shared one for client only
