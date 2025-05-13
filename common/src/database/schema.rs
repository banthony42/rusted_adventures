// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "pg_classes"))]
    pub struct PgClasses;

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
    use super::sql_types::PgClasses;

    characters (id) {
        id -> Int4,
        account_id -> Uuid,
        entity_id -> Int4,
        class -> PgClasses,
    }
}

diesel::table! {
    entities (id) {
        id -> Int4,
        #[max_length = 12]
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Point;

    locations (entity_id) {
        entity_id -> Int4,
        world -> Point,
        map -> Nullable<Point>,
    }
}

diesel::joinable!(characters -> accounts (account_id));
diesel::joinable!(characters -> entities (entity_id));
diesel::joinable!(locations -> entities (entity_id));

diesel::allow_tables_to_appear_in_same_query!(accounts, characters, entities, locations,);
