// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "classes"))]
    pub struct Classes;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "entity"))]
    pub struct Entity;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "point", schema = "pg_catalog"))]
    pub struct Point;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "races"))]
    pub struct Races;
}

diesel::table! {
    accounts (login) {
        #[max_length = 12]
        login -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 255]
        session_token -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Races;
    use super::sql_types::Classes;

    player (name) {
        #[max_length = 12]
        name -> Varchar,
        race -> Races,
        class -> Classes,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Point;

    playerlocation (name) {
        #[max_length = 12]
        name -> Varchar,
        w_coord -> Point,
        m_coord -> Point,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Point;
    use super::sql_types::Entity;

    world (coord_str) {
        #[max_length = 9]
        coord_str -> Varchar,
        w_coord -> Point,
        entity -> Entity,
    }
}

diesel::allow_tables_to_appear_in_same_query!(accounts, player, playerlocation, world,);
