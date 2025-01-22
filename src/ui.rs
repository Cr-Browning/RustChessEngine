use crate::position::{Position, Color, PieceType};
use crate::search::Search;
use crate::Game;
use crate::evaluation::Evaluation;
use crate::utils::{bit_scan, bit_scan_safe};
use std::io::{self, Write};

pub struct ChessUI {
    game: Game,
    search: Search,
    player_color: Color,
    invalid_moves: Vec<u64>,  // Track invalid moves for current turn
}

impl ChessUI {
    pub fn new() -> Self {
        ChessUI {
            game: Game::new(),
            search: Search::new(),
            player_color: Color::White,
            invalid_moves: Vec::new(),
        }
    }

    fn validate_engine_move(&self, position: &Position, engine_move: u64) -> Result<(), String> {
        let from_square = engine_move & 0x3F;
        let to_square = (engine_move >> 6) & 0x3F;
        let from_bitboard = 1u64 << from_square;

        // Verify correct turn order
        let engine_color = if self.player_color == Color::White { Color::Black } else { Color::White };
        if position.active_color != engine_color {
            return Err(format!("Engine tried to move when it's {}'s turn", 
                if position.active_color == Color::White { "White" } else { "Black" }));
        }

        // Check if there's a piece at the source square
        let piece = position.pieces.iter().find(|p| p.position == from_bitboard);
        
        if let Some(piece) = piece {
            // Verify the piece belongs to the engine
            if piece.color != engine_color {
                let move_str = self.format_move(from_square, to_square, piece.piece_type);
                let piece_name = match piece.piece_type {
                    PieceType::King => "King",
                    PieceType::Queen => "Queen",
                    PieceType::Rook => "Rook",
                    PieceType::Bishop => "Bishop",
                    PieceType::Knight => "Knight",
                    PieceType::Pawn => "Pawn",
                };
                return Err(format!("Engine tried to move {}'s {} ({})", 
                    if piece.color == Color::White { "White" } else { "Black" },
                    piece_name, 
                    move_str));
            }

            // For pawns, verify they're not moving diagonally unless capturing
            if piece.piece_type == PieceType::Pawn {
                let from_file = from_square % 8;
                let to_file = to_square % 8;
                
                if from_file != to_file {
                    // Diagonal move - must be a capture
                    let to_bitboard = 1u64 << to_square;
                    let has_enemy_piece = match piece.color {
                        Color::White => position.black_occupancy & to_bitboard != 0,
                        Color::Black => position.white_occupancy & to_bitboard != 0,
                    };
                    if !has_enemy_piece {
                        let move_str = self.format_move(from_square, to_square, piece.piece_type);
                        return Err(format!("Engine tried illegal pawn capture {} (no piece to capture)", move_str));
                    }
                }
            }
        } else {
            let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
            let from_file = files[(from_square % 8) as usize];
            let from_rank = (from_square / 8) + 1;
            return Err(format!("Engine tried to move from empty square {}{}", from_file, from_rank));
        }

        Ok(())
    }

    fn make_engine_move(&mut self, position: &mut Position) -> bool {
        println!("Engine is thinking...");
        let mut attempts = 0;
        const MAX_ATTEMPTS: i32 = 10;  // Limit retries

        while attempts < MAX_ATTEMPTS {
            let mut position_copy = position.clone();
            if let Some(engine_move) = self.search.find_best_move(&mut position_copy) {
                // Skip if this move was already found to be invalid
                if self.invalid_moves.contains(&engine_move) {
                    attempts += 1;
                    continue;
                }

                match self.validate_engine_move(position, engine_move) {
                    Ok(()) => {
                        let (from_square, to_square) = self.decode_move(engine_move);
                        if let Some(piece_type) = position.get_piece_type_at(1u64 << from_square) {
                            let eval = self.get_evaluation(position);
                            println!("Engine plays: {} ({:+.2})", 
                                self.format_move(from_square, to_square, piece_type),
                                eval as f32 / 100.0
                            );
                            position.make_move(engine_move);
                            self.display_board(position);
                            self.invalid_moves.clear();  // Clear invalid moves after successful move
                            return true;
                        } else {
                            println!("Invalid engine move: piece not found at source square");
                            self.invalid_moves.push(engine_move);
                            attempts += 1;
                            continue;
                        }
                    },
                    Err(e) => {
                        println!("Invalid engine move: {} - Retrying...", e);
                        self.invalid_moves.push(engine_move);
                        attempts += 1;
                        if attempts >= MAX_ATTEMPTS {
                            println!("Engine failed to find a valid move after {} attempts.", MAX_ATTEMPTS);
                            return false;
                        }
                        continue;
                    }
                }
            } else {
                println!("Engine could not find a move!");
                return false;
            }
        }
        false
    }

    pub fn play_game(&mut self) {
        println!("Welcome to RustChess!");
        
        // Get player color preference
        print!("Would you like to play as White or Black? (w/b): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        self.player_color = match input.trim().to_lowercase().as_str() {
            "b" | "black" => {
                println!("You are playing as Black. Engine will play as White.");
                Color::Black
            },
            _ => {
                println!("You are playing as White.");
                Color::White
            }
        };

        println!("\nEnter moves in algebraic notation (e.g., 'e2e4', 'g1f3')");
        println!("Type 'quit' to exit, 'board' to display the current position\n");

        let mut position = Position::new(&self.game);
        self.display_board(&position);
        
        // If engine plays White, make first move
        if self.player_color == Color::Black {
            let mut position_copy = position.clone();
            if let Some(engine_move) = self.search.find_best_move(&mut position_copy) {
                match self.validate_engine_move(&position, engine_move) {
                    Ok(()) => {
                        let (from_square, to_square) = self.decode_move(engine_move);
                        let eval = self.get_evaluation(&position);
                        println!("Engine plays: {} ({:+.2})", 
                            self.format_move(from_square, to_square, position.get_piece_type_at(1u64 << from_square).unwrap_or(PieceType::Pawn)),
                            eval as f32 / 100.0
                        );
                        position.make_move(engine_move);
                        self.display_board(&position);
                    },
                    Err(e) => {
                        println!("Invalid engine move: {}", e);
                        return;
                    }
                }
            }
        }
        
        loop {
            position.update_all_legal_moves(&self.game);
            
            // Check for checkmate/stalemate
            if position.get_all_legal_moves(&self.game).is_empty() {
                if position.is_in_check(&self.game) {
                    println!("\nCheckmate! {} wins!", if position.active_color == Color::White { "Black" } else { "White" });
                } else {
                    println!("\nStalemate! Game is drawn.");
                }
                break;
            }

            if position.active_color == self.player_color {
                // Player's turn
                match self.get_player_move(&position) {
                    Ok(mov) => {
                        let (from_square, to_square) = self.decode_move(mov);
                        let eval = self.get_evaluation(&position);
                        println!("Player plays: {} ({:+.2})", 
                            self.format_move(from_square, to_square, position.get_piece_type_at(1u64 << from_square).unwrap_or(PieceType::Pawn)),
                            eval as f32 / 100.0
                        );
                        position.make_move(mov);
                        self.display_board(&position);
                    }
                    Err(e) => {
                        if !e.is_empty() {
                            println!("Invalid move: {}", e);
                        }
                        continue;
                    }
                }
            } else {
                // Engine's turn
                self.invalid_moves.clear();  // Clear invalid moves at start of turn
                if !self.make_engine_move(&mut position) {
                    println!("Engine resigned!");
                    break;
                }
            }
        }
    }

    fn get_player_move(&self, position: &Position) -> Result<u64, String> {
        // Verify correct turn order
        if position.active_color != self.player_color {
            return Err(format!("It's {}'s turn to move", 
                if position.active_color == Color::White { "White" } else { "Black" }));
        }

        print!("Your move: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        let input = input.trim().to_lowercase();
        match input.as_str() {
            "quit" => std::process::exit(0),
            "board" => {
                self.display_board(position);
                return Err("".to_string());
            }
            _ => self.parse_move(&input, position)
        }
    }

    fn parse_move(&self, input: &str, position: &Position) -> Result<u64, String> {
        if input.len() != 4 {
            return Err("Move must be in format 'e2e4'".to_string());
        }

        let chars: Vec<char> = input.chars().collect();
        
        let from_file = (chars[0] as u8).wrapping_sub(b'a');
        let from_rank = (chars[1] as u8).wrapping_sub(b'1');
        let to_file = (chars[2] as u8).wrapping_sub(b'a');
        let to_rank = (chars[3] as u8).wrapping_sub(b'1');

        if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
            return Err("Invalid square".to_string());
        }

        let from_square = (from_rank * 8 + from_file) as u64;
        let to_square = (to_rank * 8 + to_file) as u64;

        // Verify piece ownership
        let from_bitboard = 1u64 << from_square;
        if let Some(piece) = position.pieces.iter().find(|p| p.position == from_bitboard) {
            if piece.color != self.player_color {
                return Err(format!("That's not your piece to move"));
            }
        } else {
            return Err("No piece at source square".to_string());
        }

        // Verify the move is legal
        let legal_moves = position.get_all_legal_moves(&self.game);
        let mov = from_square | (to_square << 6);
        
        if !legal_moves.contains(&mov) {
            return Err("Illegal move".to_string());
        }

        Ok(mov)
    }

    fn format_move(&self, from: u64, to: u64, piece_type: PieceType) -> String {
        let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let piece_symbol = match piece_type {
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Rook => "R",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Pawn => "",
        };
        
        let from_file = files[(from % 8) as usize];
        let from_rank = (from / 8) + 1;
        let to_file = files[(to % 8) as usize];
        let to_rank = (to / 8) + 1;
        
        format!("{}{}{}{}{}", piece_symbol, from_file, from_rank, to_file, to_rank)
    }

    fn decode_move(&self, mov: u64) -> (u64, u64) {
        let from_square = mov & 0x3F;
        let to_square = (mov >> 6) & 0x3F;
        (from_square, to_square)
    }

    fn get_evaluation(&self, position: &Position) -> i32 {
        let eval = Evaluation::new(position.clone());
        eval.evaluate_position()
    }

    fn display_board(&self, position: &Position) {
        println!("\n  +-----------------+");
        for rank in (0..8).rev() {
            print!("{} |", rank + 1);
            for file in 0..8 {
                let square = rank * 8 + file;
                let piece = position.pieces.iter()
                    .find(|p| bit_scan_safe(p.position).map_or(false, |pos| pos == square));
                
                let symbol = if let Some(piece) = piece {
                    match (piece.piece_type, piece.color) {
                        (PieceType::Pawn, Color::White) => "P",
                        (PieceType::Knight, Color::White) => "N",
                        (PieceType::Bishop, Color::White) => "B",
                        (PieceType::Rook, Color::White) => "R",
                        (PieceType::Queen, Color::White) => "Q",
                        (PieceType::King, Color::White) => "K",
                        (PieceType::Pawn, Color::Black) => "p",
                        (PieceType::Knight, Color::Black) => "n",
                        (PieceType::Bishop, Color::Black) => "b",
                        (PieceType::Rook, Color::Black) => "r",
                        (PieceType::Queen, Color::Black) => "q",
                        (PieceType::King, Color::Black) => "k",
                    }
                } else {
                    "."
                };
                print!(" {}", symbol);
            }
            println!(" |");
        }
        println!("  +-----------------+");
        println!("    a b c d e f g h\n");
    }
} 