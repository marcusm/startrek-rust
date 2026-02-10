# Star Trek (Rust)

A Rust implementation of the classic 1971 Star Trek text game by Mike Mayfield. Command the USS Enterprise on a mission to destroy all Klingon battle cruisers before time runs out.

## Building and Running

Build the project:
```bash
cargo build
```

Run the game:
```bash
cargo run
```

Run with a specific seed (for reproducible games):
```bash
cargo run -- --seed 12345
```

Run tests:
```bash
cargo test
```

## In-Game Commands

| Command | Action |
|---------|--------|
| 0 | Set Course (warp navigation) |
| 1 | Short Range Sensor Scan |
| 2 | Long Range Sensor Scan |
| 3 | Fire Phasers |
| 4 | Fire Photon Torpedoes |
| 5 | Shield Control |
| 6 | Damage Control Report |
| 7 | Library Computer |
| q | Quit |

## Project Structure

```
src/
├── main.rs                  # Entry point, title screen, instructions
├── lib.rs                   # Module exports
├── cli/
│   └── args.rs              # Command-line argument parsing (--seed)
├── game_engine/
│   └── mod.rs               # Game state machine, victory/defeat logic
├── io/
│   └── mod.rs               # I/O abstraction (terminal + mock for tests)
├── models/
│   ├── constants.rs         # Game constants
│   ├── position.rs          # Quadrant and sector coordinates
│   ├── enterprise.rs        # Enterprise ship state and methods
│   ├── klingon.rs           # Klingon enemy state
│   ├── quadrant.rs          # Quadrant data (klingons, starbases, stars)
│   ├── sector_map.rs        # Sector grid display
│   ├── errors.rs            # Error types
│   ├── navigation_types.rs  # Navigation type definitions
│   └── galaxy/
│       ├── mod.rs           # Galaxy struct (top-level game state)
│       ├── generation.rs    # Procedural galaxy generation
│       └── quadrant_ops.rs  # Quadrant entry and memory operations
├── services/
│   ├── game.rs              # Main game loop and command dispatch
│   ├── scan.rs              # Short and long range sensor scans
│   ├── computer.rs          # Library computer functions
│   ├── navigation/
│   │   ├── course.rs        # Course calculation
│   │   ├── movement.rs      # Warp travel and movement
│   │   └── damage.rs        # Device damage and repair
│   └── combat/
│       ├── phasers.rs       # Phaser attacks
│       ├── torpedoes.rs     # Photon torpedoes
│       ├── shields.rs       # Shield control
│       └── klingon_attack.rs # Klingon attack logic
├── ui/
│   └── presenters.rs        # Display formatting
tests/
├── integration_tests.rs     # Integration tests
└── property_tests.rs        # Property-based tests (proptest)
```

## Documentation

- [Game Specification](docs/StarTrekSpec.md) — Complete specification for the 1971 Star Trek game
- [Help Text](docs/help.txt) — In-game instructions and command reference
