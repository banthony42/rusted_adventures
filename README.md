<h1 align="center">
  <br>
  <a href="#"><img src="images/logo.png" alt="PROJECT_NAME" width="200"></a>
  <br>
  PROJECT_NAME
  <br>
</h1>

<h4 align="center">A minimal multiplayer role playing game <span style="font-weight: 750">draft</span> build with <a href="https://www.rust-lang.org/fr" target="_blank">Rust</a> and drawn with <a href="https://www.aseprite.org/" target="_blank">Aseprite</a>.</h4>


<p align="center">
  <a href="https://www.rust-lang.org/fr">
    <img src="https://img.shields.io/badge/Rust-grey?style=for-the-badge&logo=rust&logoColor=white"
         alt="Rust">
  </a>
  <a href="https://www.aseprite.org">
    <img src="https://img.shields.io/badge/Aseprite-grey?style=for-the-badge&logo=aseprite&logoColor=white"
         alt="Aseprite">
  </a>
  <a href="https://www.dofus.com/fr/mmorpg/decouvrir">
    <img src="https://img.shields.io/badge/humbly_inspired_by_dofus-83af1f?style=for-the-badge&logo=egghead&logoColor=white"
         alt="Humbly inspired by Dofus">
  </a>
</p>

<p align="center">
  <a href="#introduction">Introduction</a> |
  <a href="#description">Description</a> |
  <a href="#architecture">Architecture</a> |
  <a href="#getting-started">Getting Started</a> |
  <a href="#known-issues">Known Issues</a> |
  <a href="#credits">Credits</a>
</p>

![demo](images/demo.gif)

## Introduction
**First goal of this project is to learn !**<br>
All technologies were chosen for the sole purpose of **learning** them.<br>

Goal is not to develop following the video games state of the art.<br>
Neither to create a real game.

## Description
Welcome to PROJECT_NAME !
A minimal multiplayer role playing game, humbly inspired by [Dofus](https://www.dofus.com/fr/mmorpg/decouvrir). <br>
Here you won't spend hours killing monsters, collecting rare loot or delves perilous dungeons.<br>
Instead you can observe a passionate trying to implement it.<br>

I set myself the goal of creating the minimum of an RPG:
- [x] Game world with several maps
- [x] Player account
- [x] Player movements
- [x] Player persistency in the game
- [x] Monsters movements
- [x] Monsters persistency in the game
- [x] Chat
- [ ] Character classes
- [ ] Fight
- [ ] Levels and experience points
- [ ] Loot
- [ ] Crafts
- [ ] Gameplay loop

I have deliberately forget quests. :kissing_smiling_eyes:

## Architecture
This project is split in four part :

* Client<br>
  Displays the game state and transmits the player's actions to the server.
---
* Server<br>
  Handle all events in the game. (entities lifecycle, players input, etc ...)
---
* Database<br>
  Save accounts and players data.
---
* Command Line Interface<br>
  Also known as **CLI**, to replace the game web site and allow us to register  player accounts and manage it.
---

<p>
  <h3>Build with</h3>
  <p align="left">
      <a href="https://www.rust-lang.org/fr">
      <img src="https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust&logoColor=White"alt="Rust"></a>
      <a href="https://grpc.io/">
      <img src="https://img.shields.io/badge/gRPC-426E73?style=for-the-badge&logo=data:image/png;base64,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"alt="Rust"></a>
      <a href="https://www.postgresql.org/">
      <img src="https://img.shields.io/badge/postgresql-335781?style=for-the-badge&logo=postgresql&logoColor=white"alt="Postgresql"></a>
      <a href="https://www.aseprite.org">
      <img src="https://img.shields.io/badge/Aseprite-583E46?style=for-the-badge&logo=aseprite&logoColor=white"alt="Rust"></a>
  </p>
</p>

```mermaid
block-beta
  columns 3
  Client_1 Empty["..."] Client_n
  space:2
  space:2 Server["Server"] CLI
  space:3
  block:group1:3
  DockerLabel["Docker"]
  Database[("PostgreSQL")] space
end

  Client_1 -- "gRPC" --> Server
  Client_n -- "gRPC" --> Server
  Server -- "gRPC" --> Client_1
  Server -- "gRPC" --> Client_n

  Server -- "diesel" --> Database
  Database -- "diesel" --> Server
  CLI -- "diesel" --> Database
  Database -- "diesel" --> CLI

style Empty fill:none,stroke:none
style DockerLabel fill:none,stroke:none
```

## Getting started

### Prerequisites

```sh
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

```sh
# Populate the database
cd common/src/database && diesel migration run --migration-dir ./migrations
```

### Run

```sh
# Launch the server
cd rpg
./target/debug/server
```

```sh
# Create your player account
./target/debug/db_cli account create mylogin mypassword
```

```sh
cd client
# Mandatory to launch from this folder from now ...
../target/debug/rpg-client
```

Use your login and password account to enter in the game.

Enjoy !

## Known Issues

## Credits
- [Rust crates](https://crates.io/) : [ [serde](https://serde.rs/) - [piston](https://www.piston.rs/) - [diesel](https://diesel.rs/) - [tokio](https://tokio.rs/) - [tonic](https://github.com/hyperium/tonic) - [prost](https://crates.io/crates/prost) - [argon2](https://crates.io/crates/argon2) - [clap](https://crates.io/crates/clap) ]
- [refactoring.guru: Rust design patterns](https://refactoring.guru/design-patterns/behavioral-patterns)
- [zerotomastery: Rust type state pattern](https://zerotomastery.io/blog/rust-typestate-patterns/)
- [dev.to: Khaled Hosseini: Play Microservices Auth service](https://dev.to/khaledhosseini/play-microservices-authentication-4di3)
- [dev.to: Neeraj Sharma: Auth API in Rust using gRPC](https://dev.to/neeraj_sharma_1135657c7f6/how-to-build-an-auth-api-in-rust-grpc-57mc)
- [Jay Butera: Game server in 150 lines of Rust]() : [sources](https://github.com/jaybutera/mmo-rust-server)
- [Akanoa - La Forge](https://lafor.ge/)
- [AStar algorithm](https://www.youtube.com/watch?v=JcYyO14F6KY)
- [Owasp - Password storage cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Owasp - Authentication cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html#authentication-cheat-sheet)
- [Display resolution](https://en.wikipedia.org/wiki/Display_resolution)
- [AdamCYounis - PixelArt class](https://www.youtube.com/playlist?list=PLLdxW--S_0h4dlWUpl-TzBp-ulqK3NiM_)
- [Baba Des bois - PixelArt - Tutoriels](https://www.youtube.com/playlist?list=PLeeK5VJQ55mOjXTK2kgpoEX-JqFD03brj)
- [David Capello - export-aseprite-file](https://github.com/dacap/export-aseprite-file)