## Command Line Interface
  Also known as **CLI**, to replace the game web site and allow us to register player accounts and manage it.
  I also use it as a playground to quickly tests things.
  For example, it help me to tests some gRPC request at the begining of the project.

  ```bash
  Usage: rpg-cli <COMMAND>

Commands:
  account    Create, update, delete or show accounts
  grpc       Test GRPc services
  character  Create, delete or show characters
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
  ```

### Account management
  ```bash
  Create, update, delete or show accounts

Usage: rpg-cli account <COMMAND>

Commands:
  create  Create an new account
  update  Update an existing account
  delete  Delete an account
  show    Show all accounts
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
  ```

### Character management
  ```bash
  Create, delete or show characters

Usage: rpg-cli character <COMMAND>

Commands:
  create  Create an new character
  delete  Delete a character
  show    Show all characters of a given account (by login)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
  ```

### gRPC services

  ```bash
  Test GRPc services

Usage: rpg-cli grpc <COMMAND>

Commands:
  chat  
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help


Usage: rpg-cli grpc chat <LOGIN>

Arguments:
  <LOGIN>  The sender name

Options:
  -h, --help  Print help
  ```