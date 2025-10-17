# RPG common

This library gather shared code between other parts of the projects.
For example gRPC generated code is used both by client and server,
database queries are used by the server but also by the **CLI**, etc.

## gRPC

All the Protocol Buffer definitions leaved in [proto](proto) folder.
All gRPC services definition file should be report in [build.rs](common/build.rs) file.
And that's it.

Just update or write new services within `proto` folder.
You can get some help from the [protocol buffer documentation](https://protobuf.dev/).
Then update the gRPC generated code building the library:

```sh
cd common
cargo build
```

A folder named `grpc_codegen` is created, or updated.
You can explore the generated rust code in it, and start develop your services.
For example, you can find chat service here: (rpc stream)

- Server part : [services/chat.rs](server/src/services/chat.rs)
- Client part : [chat/client.rs](client/src/chat/client.rs)

Another example with player connection: (classic rpc request)

- Server part: [authenticate.rs](server/src/services/authenticate.rs)
- Client part: [connection.rs](client/src/tasks/connection.rs)

## PGSQL and Diesel

### Prerequisites:

```
# Windows:
# Visit https://www.postgresql.org/download/windows/ to install Postgres
# Update Windows Path with postgres bin and lib folder

# Linux:
sudo apt-get update
sudo apt-get install libpq-dev

cargo install diesel_cli --no-default-features --features postgres
```

### Setup

```sh
docker pull postgres
docker run --name my_db_name -p 4242:5432 -e POSTGRES_USER=username -e POSTGRES_PASSWORD=password -d postgres
```

```sh
git clone https://github.com/banthony42/rpg.git
echo "DATABASE_URL=postgres://username:password@localhost:4242/my_db_name" > ./rpg/.env
cd rpg && cargo build
```

### Initial migration

```sh
cd common/src/database
diesel setup
```

Diesel will create for you the migrations folder and then `YYYY-MM-DD-HHMMSS_diesel_initial_setup` folder in it.

This step is purely informational, it's used at the very beginning of a new project. It should not be used anymore here since initial migration already exist.<br>
Classic database interventions are describe in following sections:

- <a href="#create-new-migration">Create new migration</a>
- <a href="#revert-a-migration">Revert a migration</a>
- <a href="#update-a-migration">Update a migration</a>

### Create new migration

```sh
cd common/src/database
diesel migration generate my_new_migration
```

Diesel will create `migrations/YYYY-MM-DD-HHMMSS_my_new_migration` folder, with empty `up.sql` `down.sql` files in it. You are ready to edit them and begin your database schema.
When you want to test your migration :

```sh
cd common/src/database
diesel migration run
```

This will create or update `schema.rs` file.

```sh
# Test your down.sql file running this
# It revert your migration applying down.sql then run it again with up.sql
cd common/src/database
diesel migration redo
```

You're good to write rust code to play with your new schema.

### Revert a migration

```sh
cd common/src/database
diesel migration revert
```

This apply the down.sql file of the last migration, therefore the database and your schema.rs don't have any data related to the last migration. Useful during development when you have to update your migration. See <a href="#update-a-migration">Update a migration</a> section for complete update workflow.

### Update a migration

During your migration development journey you may need to edit the code you wrote in your `up.sql` `down.sql` files. Therefore you also need to update your `schema.rs` accordingly.

```sh
# First revert your migration
cd common/src/database
diesel migration revert

# You are now ready to update up.sql and down.sql

# When you are ready just run your migration again
diesel migration run

# schema.rs has been updated, update your code

# When you are happy with your modifications
# Ensure your down.sql file is still valid
diesel migration redo
```
