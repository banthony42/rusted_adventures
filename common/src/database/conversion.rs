use diesel_geometry::data_types::PgPoint;

use crate::{database::model::location::Location, CellCoord, MapCoord, Position};

impl From<PgPoint> for CellCoord {
    fn from(value: PgPoint) -> Self {
        CellCoord {
            x: value.0 as i64,
            y: value.1 as i64,
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

impl From<Location> for Position {
    fn from(value: Location) -> Self {
        Position {
            map: value.map.into(),
            cell: value.cell.into(),
        }
    }
}
