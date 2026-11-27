use diesel_geometry::data_types::PgPoint;

use crate::{
    database::model::location::UpdateLocation,
    entities::{
        model::EntityModel,
        monsters::{Monster, MonsterType},
        player::{Player, PlayerType},
    },
    grpc_codegen::{
        client_entity_event::{self, Event},
        entity::Family,
        Classes, ClientEntityEvent, Coord, Entity, Location, RequestPath, Species,
    },
    network::events::{EntityEvent, EntityEventSequences, EventKind},
    CellCoord, MapCoord, Position,
};

#[derive(Debug, thiserror::Error)]
pub enum NetConversionError {
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

    #[error("Missing Entity event, can't convert")]
    MissingEntityEvent,

    #[error("Map coordinates are out of range, can't build Position.")]
    InvalidMapCoordinates,
}

/*
** Locations, Map and Cell coordinates
*/

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

impl TryFrom<Option<Coord>> for CellCoord {
    type Error = NetConversionError;

    fn try_from(value: Option<Coord>) -> Result<Self, Self::Error> {
        match value {
            Some(coord) => Ok(coord.into()),
            None => Err(NetConversionError::MissingCell),
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
    type Error = NetConversionError;

    fn try_from(value: Coord) -> Result<Self, Self::Error> {
        Ok(Self {
            x: i8::try_from(value.x).map_err(|_| NetConversionError::InvalidMapCoordinates)?,
            y: i8::try_from(value.y).map_err(|_| NetConversionError::InvalidMapCoordinates)?,
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
    type Error = NetConversionError;

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        let map = value.map.ok_or(NetConversionError::MissingMap)?;
        let cell = value.cell.ok_or(NetConversionError::MissingCell)?;

        Ok(Self {
            map: MapCoord::try_from(map)?,
            cell: CellCoord::from(cell),
        })
    }
}

impl TryFrom<Option<Location>> for Position {
    type Error = NetConversionError;

    fn try_from(value: Option<Location>) -> Result<Self, Self::Error> {
        match value {
            Some(location) => Position::try_from(location),
            None => Err(NetConversionError::MissingLocation),
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
    type Error = NetConversionError;

    fn try_from(value: Location) -> Result<Self, Self::Error> {
        let map = value.map.ok_or(NetConversionError::MissingMap)?;
        let cell = value.cell.ok_or(NetConversionError::MissingCell)?;

        Ok(UpdateLocation {
            map: map.into(),
            cell: cell.into(),
        })
    }
}

impl TryFrom<Option<Location>> for UpdateLocation {
    type Error = NetConversionError;

    fn try_from(value: Option<Location>) -> Result<Self, Self::Error> {
        match value {
            Some(location) => UpdateLocation::try_from(location),
            None => Err(NetConversionError::MissingLocation),
        }
    }
}

/*
** Entities
*/

impl TryFrom<Entity> for PlayerType {
    type Error = NetConversionError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        match value.family {
            None => Err(NetConversionError::MissingFamily),
            val if val == Some(Family::Class(Classes::Warrior as i32)) => {
                Ok(Self::Warrior(Player::new(value.try_into()?)))
            }
            val if val == Some(Family::Class(Classes::Mage as i32)) => {
                Ok(Self::Mage(Player::new(value.try_into()?)))
            }
            Some(Family::Class(c)) => Err(NetConversionError::ClassNotFound(c)),
            Some(Family::Species(s)) => Err(NetConversionError::UnexpectedSpeciesAsPlayer(s)),
        }
    }
}

impl TryFrom<Entity> for MonsterType {
    type Error = NetConversionError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        match value.family {
            None => Err(NetConversionError::MissingFamily),
            val if val == Some(Family::Species(Species::Bouftou as i32)) => {
                Ok(Self::Bouftou(Monster::new(value.try_into()?)))
            }
            val if val == Some(Family::Species(Species::Crabedoeuf as i32)) => {
                Ok(Self::Crabedoeuf(Monster::new(value.try_into()?)))
            }
            Some(Family::Species(s)) => Err(NetConversionError::SpeciesNotFound(s)),
            Some(Family::Class(c)) => Err(NetConversionError::UnexpectedClassAsMonster(c)),
        }
    }
}

impl TryFrom<Entity> for EntityModel {
    type Error = NetConversionError;

    fn try_from(value: Entity) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.name,
            value.uuid,
            value.location.try_into()?,
        ))
    }
}

/*
** Network Events
*/

impl TryFrom<client_entity_event::Event> for EntityEvent {
    type Error = NetConversionError;

    fn try_from(value: client_entity_event::Event) -> Result<Self, Self::Error> {
        match value {
            Event::RequestedPath(request) => Ok(EntityEvent::RequestPath(
                request.location.try_into()?,
                request.destination.try_into()?,
            )),
        }
    }
}

impl TryFrom<Option<client_entity_event::Event>> for EntityEvent {
    type Error = NetConversionError;

    fn try_from(value: Option<client_entity_event::Event>) -> Result<Self, Self::Error> {
        match value {
            Some(event) => event.try_into(),
            None => Err(NetConversionError::MissingEntityEvent),
        }
    }
}

pub trait IntoClientEntityEvent {
    fn into_net_event(
        self,
        sequences: &mut EntityEventSequences,
    ) -> Result<ClientEntityEvent, NetConversionError>;
}

impl IntoClientEntityEvent for EntityEvent {
    fn into_net_event(
        self,
        sequences: &mut EntityEventSequences,
    ) -> Result<ClientEntityEvent, NetConversionError> {
        Ok(match self {
            EntityEvent::RequestPath(position, destination) => ClientEntityEvent {
                event: Some(Event::RequestedPath(RequestPath {
                    sequence_number: sequences.next(EventKind::RequestPath),
                    location: Some(position.into()),
                    destination: Some(destination.into()),
                })),
            },
        })
    }
}
