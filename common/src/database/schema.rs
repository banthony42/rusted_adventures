// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (login) {
        #[max_length = 12]
        login -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 255]
        session_token -> Varchar,
    }
}
