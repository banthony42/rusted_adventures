<h1 align="center">
  <img src="images/logo.png" alt="RPG" width="192">
  <div>RPG</div>
</h1>

<h4 align="center">A minimal multiplayer role playing game <span style="font-weight: 750">draft</span> build with <a href="https://www.rust-lang.org/fr" target="_blank" rel="noopener noreferrer">Rust</a> and drawn with <a href="https://www.aseprite.org/" target="_blank" rel="noopener noreferrer">Aseprite</a>.</h4>

<p align="center">
  <a href="https://www.rust-lang.org/fr" target="_blank" rel="noopener noreferrer">
    <img src="https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust&logoColor=White"
         alt="Rust">
  </a>
  <a href="https://www.aseprite.org" target="_blank" rel="noopener noreferrer">
    <img src="https://img.shields.io/badge/Aseprite-583E46?style=for-the-badge&logo=aseprite&logoColor=white"
         alt="Aseprite">
  </a>
  <a href="https://www.dofus-retro.com/en/mmorpg/discover" target="_blank" rel="noopener noreferrer">
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

<p align="center">
  <img src="images/demo.gif" alt="animated" />
</p>

## Introduction

**First goal of this project is to learn !**<br>
All technologies were chosen for the sole purpose of **learning** them, even my pixel art editor.<br>

Goal is not to develop following the video games state of the art.<br>
Neither to create a real game.
Even if i'm passionate about heroic fantasy games such as Dofus, World of Warcraft or Skyrim,
it's only a pretext here, to be confront to interesting problematics to develop.

## Description

Welcome to RPG !
A minimal multiplayer role playing game, humbly inspired by [Dofus Retro](https://www.dofus-retro.com/en/mmorpg/discover).<br>
Here you won't spend hours killing monsters, collecting rare loot or delves perilous dungeons.<br>
Instead you can observe a passionate trying to implement it.<br>

I set myself the goal of creating the minimum of an RPG:

- [x] Game world with several maps
- [x] Player account
- [x] Player movements
- [x] Player persistency in the game
- [x] Monsters spawn
- [x] Monsters movements
- [x] Monsters persistency in the game
- [x] Entities events send to each concerned players
- [x] Chat (map channel and private messages)
- [ ] Character classes
- [ ] Fight
- [ ] Levels and experience points
- [ ] Loot
- [ ] Crafts
- [ ] Gameplay loop

I have deliberately forget quests. :kissing_smiling_eyes:

## Architecture

This project is split in several parts :

- Client:
  Displays the game state and transmits the player's actions to the server.

---

- Server:
  Composed of :
  - gRPC services to handle all clients events in the game. (entities lifecycle, players movements...)
  - The world engine which handle the entities behaviours, spawn ...

---

- Command Line Interface:
  Also known as **CLI**, to replace the game web site and allow us to register player accounts and manage it.
  I also use it as a playground to quickly tests things.
  For example, it help me to tests some gRPC request at the begining of the project.

---

- Common Library:
  This library contain all the code shared by all other parts. (constants, database access and queries, generated gRPC code, world maps loading ...)

---

- Database:
  Save accounts, players and entities data.

---

<p>
  <h3>Build with</h3>
  <p align="left">
      <a href="https://www.rust-lang.org/fr" target="_blank" rel="noopener noreferrer">
      <img src="https://img.shields.io/badge/Rust-black?style=for-the-badge&logo=rust&logoColor=White"alt="Rust"></a>
      <a href="https://grpc.io/" target="_blank" rel="noopener noreferrer">
      <img src="https://img.shields.io/badge/gRPC-426E73?style=for-the-badge&logo=data:image/png;base64,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"alt="gRPC"></a>
      <a href="https://www.postgresql.org/" target="_blank" rel="noopener noreferrer">
      <img src="https://img.shields.io/badge/postgresql-335781?style=for-the-badge&logo=postgresql&logoColor=white"alt="Postgresql"></a>
      <a href="https://www.aseprite.org" target="_blank" rel="noopener noreferrer">
      <img src="https://img.shields.io/badge/Aseprite-583E46?style=for-the-badge&logo=aseprite&logoColor=white"alt="Aseprite"></a>
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
# Ensure your database is running
docker start my_db_name
```

```sh
# Launch the server
cd rpg
./target/debug/server
```

```sh
# Create your player account
./target/debug/rpg-cli account create mylogin mypassword
```

```sh
# Launch the game client
# Its mandatory to launch from this folder from now ...
cd client
../target/debug/rpg-client
```

Use your login and password account to enter in the game.

Enjoy !

## Known Issues

All known issues, and tasks are tracked here : [Trello Board](https://trello.com/b/SlS6G8vq)

## Credits

#### Rust :

- [Steve Klabnik - The Rust Programming Language](https://steveklabnik.com/)
- [Luca Palmieri - Zero To Production In Rust](https://www.zero2prod.com/index.html?country=France&discount_code=VAT20&country_code=FR)
- [Akanoa - La Forge](https://lafor.ge/)
  <br>
- [Rust crates](https://crates.io/) : [ [serde](https://serde.rs/) - [piston](https://www.piston.rs/) - [diesel](https://diesel.rs/) - [tokio](https://tokio.rs/) - [tonic](https://github.com/hyperium/tonic) - [prost](https://crates.io/crates/prost) - [argon2](https://crates.io/crates/argon2) - [clap](https://crates.io/crates/clap) ]
- [refactoring.guru: Rust design patterns](https://refactoring.guru/design-patterns/behavioral-patterns)
- [zerotomastery: Rust type state pattern](https://zerotomastery.io/blog/rust-typestate-patterns/)
- [dev.to: Khaled Hosseini: Play Microservices Auth service](https://dev.to/khaledhosseini/play-microservices-authentication-4di3)
- [dev.to: Neeraj Sharma: Auth API in Rust using gRPC](https://dev.to/neeraj_sharma_1135657c7f6/how-to-build-an-auth-api-in-rust-grpc-57mc)
- [protocol buffer documentation](https://protobuf.dev/)

#### Pixel Art :

- [AdamCYounis - PixelArt class](https://www.youtube.com/playlist?list=PLLdxW--S_0h4dlWUpl-TzBp-ulqK3NiM_)
- [Baba Des bois - PixelArt - Tutoriels](https://www.youtube.com/playlist?list=PLeeK5VJQ55mOjXTK2kgpoEX-JqFD03brj)
- [David Capello - export-aseprite-file](https://github.com/dacap/export-aseprite-file)

#### Security :

- [Owasp - Password storage cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Owasp - Authentication cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html#authentication-cheat-sheet)

#### Game dev :

- [Display resolution](https://en.wikipedia.org/wiki/Display_resolution)
- [AStar algorithm](https://www.youtube.com/watch?v=JcYyO14F6KY)
- [Jay Butera: Game server in 150 lines of Rust]() : [sources](https://github.com/jaybutera/mmo-rust-server)
