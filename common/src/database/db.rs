use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;

pub struct Database {
    url: String,
}

impl Database {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/database/migrations");

    pub fn new() -> Self {
        dotenv().ok();

        let url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                println!("DATABASE_URL env variable not set.");
                println!("Trying to build DATABASE_URL loading: POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_HOST, POSTGRES_DB");

                let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set.");
                let password =
                    env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set.");
                let db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set.");
                let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set.");

                format!("postgres://{user}:{password}@{host}:5432/{db}")
            }
        };

        Database { url }
    }

    pub fn establish_connection(&self) -> PgConnection {
        PgConnection::establish(&self.url)
            .expect(format!("Database connection error: {}", self.url).as_str())
    }

    pub fn run_migration(&self) -> Result<(), String> {
        let mut conn: PgConnection = self.establish_connection();
        let migrations = conn
            .run_pending_migrations(Self::MIGRATIONS)
            .map_err(|e| e.to_string())?;

        for migration in &migrations {
            println!("Migration: {}", migration);
        }

        Ok(())
    }
}
