use crate::position::{Position, Color};
use crate::search::Search;
use crate::Game;
use std::io::{self, Write};

pub struct ChessUI {
    game: Game,
    search: Search,
}

impl ChessUI {
    pub fn new() -> Self {
        ChessUI {
            game: Game::new(),
            search: Search::new(),
        }
    }

    pub fn play_game(&mut self) {
        println!("Welcome to RustChess!");
        println!("Enter moves in format: e2e4");
        println!("Commands: 'quit' to exit, 'display' to show board\n");

        let mut position = Position::new(&self.game);
        
        loop {
            println!("{}", position.to_string());
            
            if position.active_color == Color::White {
                // Human's turn (White)
                match self.get_human_move() {
                    Ok(mov) => {
                        position.make_move(mov);
                    }
                    Err(e) => {
                        println!("Invalid move: {}", e);
                        continue;
                    }
                }
            } else {
                // Engine's turn (Black)
                println!("Engine is thinking...");
                match self.search.find_best_move(&mut position) {
                    Some(mov) => {
                        println!("Engine plays: {}", self.format_move(mov));
                        position.make_move(mov);
                    }
                    None => {
                        println!("Engine could not find a move!");
                        break;
                    }
                }
            }

            // Update legal moves after each move
            position.update_all_legal_moves(&self.game);

            // Check for game end conditions
            if self.is_game_over(&position) {
                break;
            }
        }
    }

    fn get_human_move(&self) -> Result<u64, String> {
        print!("Your move: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        let input = input.trim().to_lowercase();
        match input.as_str() {
            "quit" => std::process::exit(0),
            "display" => Ok(0), // Special case to redisplay the board
            _ => self.parse_move(&input),
        }
    }

    fn parse_move(&self, input: &str) -> Result<u64, String> {
        if input.len() != 4 {
            return Err("Move must be in format 'e2e4'".to_string());
        }

        let from_file = input.chars().nth(0).unwrap() as u8 - b'a';
        let from_rank = input.chars().nth(1).unwrap() as u8 - b'1';
        let to_file = input.chars().nth(2).unwrap() as u8 - b'a';
        let to_rank = input.chars().nth(3).unwrap() as u8 - b'1';

        if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
            return Err("Invalid square".to_string());
        }

        let from_square = (from_rank * 8 + from_file) as u64;
        let to_square = (to_rank * 8 + to_file) as u64;

        // Encode move (from_square in bits 0-5, to_square in bits 6-11)
        Ok(from_square | (to_square << 6))
    }

    fn format_move(&self, mov: u64) -> String {
        let from_square = mov & 0x3F;
        let to_square = (mov >> 6) & 0x3F;

        let from_file = (from_square % 8) as u8;
        let from_rank = (from_square / 8) as u8;
        let to_file = (to_square % 8) as u8;
        let to_rank = (to_square / 8) as u8;

        format!("{}{}{}{}",
            (b'a' + from_file) as char,
            (b'1' + from_rank) as char,
            (b'a' + to_file) as char,
            (b'1' + to_rank) as char
        )
    }

    fn is_game_over(&self, position: &Position) -> bool {
        // Get all legal moves
        let legal_moves = position.get_all_legal_moves(&self.game);
        
        if legal_moves.is_empty() {
            if position.is_in_check() {
                println!("{} wins by checkmate!", 
                    if position.active_color == Color::White { "Black" } else { "White" });
            } else {
                println!("Game drawn by stalemate!");
            }
            return true;
        }
        
        false
    }
} 