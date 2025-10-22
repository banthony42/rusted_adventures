use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

pub struct Database {
    url: String,
}

impl Database {
    pub fn new() -> Self {
        dotenv().ok();
        Database {
            url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }

    pub fn establish_connection(&self) -> PgConnection {
        PgConnection::establish(&self.url)
            .expect(format!("Database connection error: {}", self.url).as_str())
    }
}
