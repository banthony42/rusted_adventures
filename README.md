# client - server RPG (in Rust)

**First goal of this project is to learn Rust !**<br>
This project does not have the ambition to create a perfect RPG
or to develop a "state-of-the-art" game.
<br>
Not at all.
<br>
But i really like RPGs and it's a good project to learn a lot from.

# Description

This game should be split in four part :

- The client which display the **state** of the game. (map, entities, players).
- The server which handle all events in the game. (entities spawn and lifecycle, players, etc ...)
- The database which save accounts and players data.
- A database CLI which replace the game web site and allow us to register an account and manage it.

# Architecture

I am constantly thinking to find good trade off between dev time, technologies, best practices and learning.
<br>
Here are the current state of my goal:

```mermaid
block-beta
  columns 3
  Client_1 Empty["..."] Client_n
  space:2
  space:2 Server["Server"] DB_CLI
  space:3
  block:group1:3
  DockerLabel["Docker"]
  Database[("PostgreSQL")] space
end

  Client_1 -- "grpc" --> Server
  Client_n -- "grpc" --> Server
  Server -- "grpc" --> Client_1
  Server -- "grpc" --> Client_n

  Server -- "diesel" --> Database
  Database -- "diesel" --> Server
  DB_CLI -- "diesel" --> Database
  Database -- "diesel" --> DB_CLI

style Empty fill:none,stroke:none
style DockerLabel fill:none,stroke:none
```

- client : Game client in Rust

  - Graphics with piston
  - Assets with Aseprite (home made)
  - Communication with Server using grpc (as client)

- server : Game server in Rust (Authentication and World)

  - Commmunication with Database using an ORM (diesel)
  - Communication with client using grpc (as server)
  - Computing game states for clients

- db_cli : Rust CLI to manage users accounts.

  - Replace a web site accounts operations
  - Communication with Database using an ORM (diesel)

- database : Save users accounts and game data. (characters, etc)
  - PostgreSQL database
  - Docker image to easily manage the DB.

# Setup:

#### Database:

```
docker pull postgres
# Don't use that values, use decent values and avoid to push it to github
docker run --name my_db_name -p 4242:5432 -e POSTGRES_USER=username -e POSTGRES_PASSWORD=password -d postgres
```

#### Diesel ORM:

```
# Windows:
# Visit https://www.postgresql.org/download/windows/ to install Postgres
# Update Windows Path with postgres bin and lib folder

# Linux:
sudo apt-get update
sudo apt-get install libpq-dev

cargo install diesel_cli --no-default-features --features postgres
```

1 - Create .env file in db_cli folder and link it in server folder

```
# Update according to Install step and don't push .env file to github
DATABASE_URL=postgres://username:password@localhost:4242/my_db_name
```

```
cd server && ln -s ../db_cli/.env .
```

2 - If `common/src/database/migrations` diesel folder already exist jump to step 5

---

3 - Run diesel cli to setup everything: db, migration folder with initial migration file.

```
cd common/src/database && diesel setup
```

3 - Create the first migration file

```
cd common/src/database && diesel migration generate accounts
```

4 - Implement up.sql and down.sql, which respectively should create and drop your TABLE.<br>

---

5 - Apply your migration running:

```
cd common/src/database && diesel migration run --migration-dir ./migrations
```

In future this should be done automatically in server code.

6 - Ensure your TABLE is created in your db and ensure down.sql is also correct running this:

```
cd common/src/database && diesel migration redo --migration-dir ./migrations
```

7 - schema.rs has been created or updated at migration apply, you are ready to dev and build.

Notes:
To add new data to the Database: (without generate new migration)
* Undo the migration running: `diesel migration revert --migration-dir ./migrations` or run `last_migration/down.sql` to your database.
* Append new data to your `last_migration/up.sql` & `last_migration/down.sql`
* Run `diesel migration run --migration-dir ./migrations` (will generate the new schema.rs)
* Finally run `diesel migration redo --migration-dir ./migrations` to be sure down.sql is correct. 

## Tools

- [Rust](https://www.rust-lang.org/fr/learn) [ [serde](https://serde.rs/) - [piston](https://www.piston.rs/) - [diesel](https://diesel.rs/) - [tokio](https://tokio.rs/) - [tonic](https://github.com/hyperium/tonic)]
- [postgresql](https://www.postgresql.org/)
- [grpc](https://grpc.io/)
- [Aseprite](https://www.aseprite.org/)
