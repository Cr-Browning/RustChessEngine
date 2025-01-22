//! Chess engine main module.
//! 
//! This module serves as the entry point for the chess engine and coordinates
//! the interaction between various components such as position management,
//! move generation, and attack pattern calculation.

pub mod position;
pub mod utils;
pub mod knightattacks;
pub mod rayattacks;
pub mod movegen_tables;
pub mod movegeneration;
pub mod pawnattacks;
pub mod perft;
pub mod moveorder;
pub mod evaluation;
pub mod search;
pub mod ui;
pub mod gui;
use position::*;
use knightattacks::KnightAttacks;
use pawnattacks::PawnAttacks;
use rayattacks::Rays;
use movegen_tables::MoveGenTables;
use perft::Perft;
use gui::run_gui;

/// The main game structure that holds the current position and pre-computed tables.
/// 
/// This struct serves as the central point for managing the game state and
/// providing access to various pre-computed lookup tables used for efficient
/// move generation and position evaluation.
#[derive(Debug, Clone)]
pub struct Game {
    /// The current position of the game
    position: Position,
    /// Pre-computed knight attack patterns
    knight_attacks: KnightAttacks,
    /// Pre-computed pawn move and attack patterns
    pawn_attacks: PawnAttacks,
    /// Pre-computed ray attacks for sliding pieces
    rays: Rays,
    /// Pre-computed move generation tables
    move_gen_tables: MoveGenTables,
}

impl Game {
    /// Creates a new game instance with the standard starting position.
    /// 
    /// This function initializes all pre-computed tables and sets up
    /// the board in the standard chess starting position.
    /// 
    /// # Returns
    /// 
    /// * A new `Game` instance ready for play
    pub fn new() -> Game {
        let temp_game = Game {
            position: Position {
                pieces: vec![],
                squares: vec![],
                active_color: Color::White,
                castling_rights: CastlingRights::ALL,
                en_passant: None,
                halfmove_clock: 0,
                fullmove_number: 1,
                white_occupancy: 0,
                black_occupancy: 0,
                white_kingside_path_attacked: false,
                white_queenside_path_attacked: false,
                black_kingside_path_attacked: false,
                black_queenside_path_attacked: false,
                piece_legal_moves: vec![],
                white_king_moved: false,
                black_king_moved: false,
                white_kingside_rook_moved: false,
                white_queenside_rook_moved: false,
                black_kingside_rook_moved: false,
                black_queenside_rook_moved: false,
            },
            rays: Rays::new(),
            move_gen_tables: MoveGenTables::new(),
            pawn_attacks: PawnAttacks::new(),
            knight_attacks: KnightAttacks::new(),
        };

        Game {
            position: Position::new(&temp_game),
            rays: Rays::new(),
            move_gen_tables: MoveGenTables::new(),
            pawn_attacks: PawnAttacks::new(),
            knight_attacks: KnightAttacks::new(),
        }
    }

    /// Creates a new game instance from a FEN string.
    /// 
    /// This function allows initializing the game from any valid position
    /// specified in Forsyth–Edwards Notation (FEN).
    /// 
    /// # Arguments
    /// 
    /// * `fen` - A string containing the FEN representation of the position
    /// 
    /// # Returns
    /// 
    /// * A new `Game` instance with the specified position
    pub fn from_fen(fen: &str) -> Game {
        let game = Game::new();
        Game {
            position: Position::read_FEN(fen, &game),
            rays: Rays::new(),
            move_gen_tables: MoveGenTables::new(),
            pawn_attacks: PawnAttacks::new(),
            knight_attacks: KnightAttacks::new(),
        }
    }

    pub fn from_not_alot(not_alot: &str) -> Game {
        let game = Game::new();
        let position = Position::read_FEN(not_alot, &game);
        Game {
            position,
            rays: Rays::new(),
            move_gen_tables: MoveGenTables::new(),
            pawn_attacks: PawnAttacks::new(),
            knight_attacks: KnightAttacks::new(),
        }
    }

    pub fn perft(not_alot: &str, depth: usize) -> usize {
        let game = Game::new();
        let position = Position::read_FEN(not_alot, &game);
        let mut perft = Perft::new();
        perft.run(&position, depth as i32) as usize
    }
}

fn main() {
    if let Err(e) = run_gui() {
        eprintln!("Error running GUI: {}", e);
    }
}