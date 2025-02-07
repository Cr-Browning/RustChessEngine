# RustChess Engine

A powerful chess engine written in Rust, featuring both a command-line interface and a graphical user interface.

## Features

- Complete chess rules implementation including:
  - All piece movements (Pawns, Knights, Bishops, Rooks, Queens, Kings)
  - Special moves (En passant, Castling, Pawn promotion)
  - Check and checkmate detection
  - Legal move validation

- Advanced chess engine features:
  - Alpha-beta pruning search
  - Iterative deepening
  - Move ordering optimization
  - Transposition table
  - Piece-square tables for evaluation
  - Quiescence search
  - Material and positional evaluation

- Performance optimizations:
  - Bitboard representation
  - Pre-computed move tables
  - Magic bitboards for sliding piece attacks
  - Zobrist hashing

- User Interface:
  - Interactive command-line interface
  - Graphical user interface with drag-and-drop moves
  - FEN position import/export
  - Move history display
  - Real-time evaluation bar

## Building and Running

### Prerequisites

- Rust toolchain (1.70.0 or later)
- Cargo package manager

### Building

```bash
# Clone the repository
git clone [repository-url]
cd RustChess

# Build the project
cargo build --release
```

### Running

```bash
# Run with GUI (recommended)
cargo run --release

# Run with command-line interface
cargo run --release -- --cli
```

## Project Structure

- `src/`
  - `main.rs` - Entry point and game initialization
  - `position.rs` - Chess position representation and move generation
  - `rayattacks.rs` - Sliding piece (Bishop, Rook, Queen) attack generation
  - `movegeneration.rs` - Legal move generation logic
  - `evaluation.rs` - Position evaluation
  - `search.rs` - Alpha-beta search implementation
  - `transposition.rs` - Transposition table for search optimization
  - `gui.rs` - Graphical user interface implementation
  - `ui.rs` - Command-line interface implementation

## Technical Details

### Board Representation

The engine uses bitboards for efficient position representation and move generation:
- Each piece type and color has its own 64-bit integer
- Pre-computed attack tables for all pieces
- Efficient bit manipulation for move generation

### Search Algorithm

- Negamax with alpha-beta pruning
- Iterative deepening up to depth 4
- Move ordering:
  - Captures (MVV-LVA)
  - Promotions
  - History heuristic
- Quiescence search for tactical stability

### Evaluation

- Material counting
- Piece-square tables
- Pawn structure evaluation
- King safety
- Mobility
- Center control

## Contributing

Contributions are welcome! Please feel free to submit pull requests. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT License](LICENSE)

## Acknowledgments

- Chess Programming Wiki for various chess programming concepts and techniques
- The Rust community for excellent documentation and support

## Future Improvements

- [ ] UCI protocol support
- [ ] Opening book integration
- [ ] Endgame tablebases
- [ ] Multi-threading support
- [ ] Time management improvements
- [ ] PGN import/export
- [ ] Network play capability 
