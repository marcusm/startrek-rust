# startrek-rust
Create startrek game following spec using the rust language

## Project Structure

This is a Rust workspace with two crates:

- **startrek_lib**: A library crate containing types and business logic
  - `Starship` struct with energy and shield management
  - Methods for firing torpedoes and recharging shields

- **startrek_console**: A console application that uses the library
  - Provides a simple UI for interacting with the game

## Building and Running

Build the project:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

Run the console application:
```bash
cargo run --bin startrek_console
```
