use std::io::Write;

use rand::{
    distr::{Distribution, StandardUniform},
    RngExt,
};

use diesel::{
    allow_columns_to_appear_in_same_group_by_clause,
    deserialize::{self, FromSql, FromSqlRow},
    dsl::{exists, now},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::{AsChangeset, Insertable, Queryable},
    result::Error as DieselError,
    serialize::{self, IsNull, Output, ToSql},
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl,
    Selectable, SelectableHelper,
};

use diesel_geometry::data_types::PgPoint;
use diesel_geometry::prelude::PgSameAsExpressionMethods;

use crate::grpc_codegen::{
    entity::Family, Classes as RpcClasses, Entity as RpcEntity, Location as RpcLocation,
};

use crate::database::{
    db::DBConnection,
    model::{location::Location, EntityIdentifiable},
    schema::sql_types::Pgclass,
    schema::{accounts, characters, entities, locations, sessions},
};

allow_columns_to_appear_in_same_group_by_clause!(characters::name, locations::map);

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = Pgclass)]
pub enum PgClasses {
    Warrior,
    Mage,
}

impl From<PgClasses> for Family {
    fn from(val: PgClasses) -> Self {
        match val {
            PgClasses::Warrior => Family::Class(RpcClasses::Warrior.into()),
            PgClasses::Mage => Family::Class(RpcClasses::Mage.into()),
        }
    }
}

impl ToSql<Pgclass, Pg> for PgClasses {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PgClasses::Warrior => out.write_all(b"Warrior")?,
            PgClasses::Mage => out.write_all(b"Mage")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Pgclass, Pg> for PgClasses {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Warrior" => Ok(PgClasses::Warrior),
            b"Mage" => Ok(PgClasses::Mage),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl Distribution<PgClasses> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PgClasses {
        match rng.random_range(0..=1) {
            0 => PgClasses::Warrior,
            _ => PgClasses::Mage,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct Character {
    pub id: i32,
    pub account_id: uuid::Uuid,
    pub entity_id: i32,
    pub name: String,
    pub class: PgClasses,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct CreateCharacter {
    pub account_id: uuid::Uuid,
    pub entity_id: i32,
    pub name: String,
    pub class: PgClasses,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::characters)]
pub struct UpdateCharacter {
    pub class: PgClasses,
    pub name: String,
}

type CharacterInfoData = (i32, String, PgClasses, PgPoint, PgPoint, Option<PgPoint>);

impl From<CharacterInfoData> for CharacterInfo {
    fn from(val: CharacterInfoData) -> Self {
        let (id, name, class, map, cell, destination) = val;
        CharacterInfo {
            id,
            name,
            class,
            map,
            cell,
            destination,
        }
    }
}

pub struct CharacterInfo {
    pub id: i32,
    pub name: String,
    pub class: PgClasses,
    pub map: PgPoint,
    pub cell: PgPoint,
    pub destination: Option<PgPoint>,
}

impl CharacterInfo {
    pub fn from_character(
        db: &mut DBConnection,
        character: &Character,
    ) -> Result<CharacterInfo, DieselError> {
        let location = Location::read(db, &character.entity_id)?;
        Ok(CharacterInfo {
            id: character.id,
            name: character.name.clone(),
            class: character.class.clone(),
            map: location.map,
            cell: location.cell,
            destination: location.destination,
        })
    }
}

impl EntityIdentifiable for CharacterInfo {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl EntityIdentifiable for Character {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}

impl From<CharacterInfo> for RpcEntity {
    fn from(val: CharacterInfo) -> Self {
        RpcEntity {
            uuid: val.identifier(),
            name: val.name,
            family: Some(val.class.into()),
            location: Some(RpcLocation {
                map: Some(val.map.into()),
                cell: Some(val.cell.into()),
            }),
        }
    }
}

impl Character {
    /// Create a Character in DB with the given CreateCharacter item
    pub fn create(db: &mut DBConnection, item: &CreateCharacter) -> QueryResult<Self> {
        diesel::insert_into(characters::table)
            .values(item)
            .get_result(db)
    }

    /// Return the Character in DB for the given character id
    pub fn read(db: &mut DBConnection, id: &i32) -> QueryResult<Self> {
        characters::table
            .filter(characters::id.eq(id))
            .first::<Character>(db)
    }

    /// Return all the Character in DB
    pub fn read_all(db: &mut DBConnection) -> QueryResult<Vec<Self>> {
        characters::table.load::<Character>(db)
    }

    /// Update an Character in DB for the given character id according to the given UpdateCharacter item.
    pub fn update(db: &mut DBConnection, id: &i32, item: &UpdateCharacter) -> QueryResult<Self> {
        diesel::update(characters::table.filter(characters::id.eq(id)))
            .set(item)
            .returning(Character::as_returning())
            .get_result(db)
    }

    /// Delete Character in DB of the given character id
    pub fn delete(db: &mut DBConnection, id: &i32) -> QueryResult<usize> {
        diesel::delete(characters::table.filter(characters::id.eq(id))).execute(db)
    }

    /// Return the Character in DB for the given account login
    pub fn read_by_account(
        db: &mut DBConnection,
        account_login: &String,
    ) -> QueryResult<Vec<Self>> {
        accounts::table
            .inner_join(characters::table)
            .filter(accounts::login.eq(account_login))
            .select(Character::as_select())
            .load(db)
    }

    pub fn characters_names_group_by_map(
        db: &mut DBConnection,
        eid: i32,
    ) -> QueryResult<Vec<String>> {
        characters::table
            .inner_join(entities::table)
            .inner_join(locations::table.on(locations::id.eq(entities::location_id)))
            .group_by((locations::map, characters::name))
            .filter(entities::id.eq(eid))
            .select(characters::name)
            .load(db)
    }

    pub fn read_names_by_map_exclude(
        db: &mut DBConnection,
        map_coord: PgPoint,
        exclude: &String,
    ) -> QueryResult<Vec<String>> {
        characters::table
            .inner_join(entities::table)
            .inner_join(locations::table.on(locations::id.eq(entities::location_id)))
            .filter(locations::map.same_as(map_coord))
            .filter(characters::name.ne(exclude))
            .select(characters::name)
            .load(db)
    }

    pub fn read_by_map(
        db: &mut DBConnection,
        map_coord: PgPoint,
    ) -> QueryResult<Vec<CharacterInfo>> {
        let data: Vec<CharacterInfoData> = characters::table
            .inner_join(entities::table)
            .inner_join(locations::table.on(locations::id.eq(entities::location_id)))
            .filter(locations::map.same_as(map_coord))
            .filter(exists(
                sessions::table.filter(
                    sessions::account_id
                        .eq(characters::account_id)
                        .and(sessions::expires_at.gt(now)),
                ),
            ))
            .select((
                characters::id,
                characters::name,
                characters::class,
                locations::map,
                locations::cell,
                locations::destination,
            ))
            .load(db)?;

        Ok(data.iter().map(|data| data.to_owned().into()).collect())
    }
}
