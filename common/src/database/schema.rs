// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pgclass"))]
    pub struct Pgclass;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pgspecies"))]
    pub struct Pgspecies;
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;

    accounts (id) {
        id -> Uuid,
        #[max_length = 12]
        login -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        login_failure_count -> Int4,
        login_window_started_at -> Timestamptz,
        locked_until -> Nullable<Timestamptz>,
        lockout_count -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;
    use super::sql_types::Pgspecies;

    bestiary (id) {
        id -> Int4,
        species -> Pgspecies,
        #[max_length = 16]
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;
    use super::sql_types::Pgclass;

    characters (id) {
        id -> Int4,
        account_id -> Uuid,
        entity_id -> Int4,
        #[max_length = 16]
        name -> Varchar,
        class -> Pgclass,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;

    entities (id) {
        id -> Int4,
        location_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;

    locations (id) {
        id -> Int4,
        map -> Point,
        cell -> Point,
        destination -> Nullable<Point>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;

    monsters (bestiary_id, entity_id) {
        id -> Int4,
        bestiary_id -> Int4,
        entity_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_geometry::sql_types::*;

    sessions (id) {
        id -> Uuid,
        account_id -> Uuid,
        #[max_length = 64]
        token_hash -> Bpchar,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        last_used_at -> Timestamptz,
    }
}

diesel::joinable!(characters -> accounts (account_id));
diesel::joinable!(characters -> entities (entity_id));
diesel::joinable!(entities -> locations (location_id));
diesel::joinable!(monsters -> bestiary (bestiary_id));
diesel::joinable!(monsters -> entities (entity_id));
diesel::joinable!(sessions -> accounts (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    bestiary,
    characters,
    entities,
    locations,
    monsters,
    sessions,
);
