// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pgbestiary"))]
    pub struct Pgbestiary;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pgclass"))]
    pub struct Pgclass;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "point", schema = "pg_catalog"))]
    pub struct Point;
}

diesel::table! {
    accounts (id) {
        id -> Uuid,
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
    use super::sql_types::Pgclass;

    characters (id) {
        id -> Int4,
        account_id -> Uuid,
        entity_id -> Int4,
        class -> Pgclass,
    }
}

diesel::table! {
    entities (id) {
        id -> Int4,
        uuid -> Uuid,
        #[max_length = 16]
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Point;

    locations (entity_id) {
        entity_id -> Int4,
        world -> Point,
        map -> Point,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Pgbestiary;

    monsters (id) {
        id -> Int4,
        entity_id -> Int4,
        race -> Pgbestiary,
    }
}

diesel::joinable!(characters -> accounts (account_id));
diesel::joinable!(characters -> entities (entity_id));
diesel::joinable!(locations -> entities (entity_id));
diesel::joinable!(monsters -> entities (entity_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    characters,
    entities,
    locations,
    monsters,
);
