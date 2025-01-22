use eframe::egui;
use crate::Game;
use crate::position::{Color, PieceType};
use crate::utils::bit_scan;
use crate::evaluation::Evaluation;
use crate::search::Search;


#[derive(Clone)]
pub struct ChessGUI {
    game: Game,
    selected_square: Option<usize>,
    is_player_turn: bool,
    evaluation: i32,  // Current position evaluation in centipawns
    player_color: Color,  // Added player color field
    search: Search,  // Added search engine
    engine_thinking: bool,  // Flag to prevent multiple engine moves
}

impl ChessGUI {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            game: Game::new(),
            selected_square: None,
            is_player_turn: true,
            evaluation: 0,
            player_color: Color::White,  // Default to white
            search: Search::new(),
            engine_thinking: false,
        }
    }

    fn make_engine_move(&mut self) {
        if self.is_player_turn || self.engine_thinking {
            return;
        }

        self.engine_thinking = true;

        // Ensure engine plays the opposite color of the player
        let engine_color = if self.player_color == Color::White { Color::Black } else { Color::White };
        if self.game.position.active_color != engine_color {
            self.engine_thinking = false;
            return;
        }

        println!("Engine is thinking..."); // Debug print

        // Update legal moves before searching
        let game_copy = self.game.clone();
        self.game.position.update_all_legal_moves(&game_copy);

        println!("Active color before move: {:?}", self.game.position.active_color); // Debug print

        // Find best move using alpha-beta search
        let mut position_copy = self.game.position.clone();
        if let Some(best_move) = self.search.find_best_move(&mut position_copy) {
            // Extract from and to squares from the move
            let from_square = (best_move & 0x3F) as usize;
            let to_square = ((best_move >> 6) & 0x3F) as usize;
            
            // Convert squares to chess notation
            let from_file = (from_square % 8) as u8;
            let from_rank = (from_square / 8) as u8;
            let to_file = (to_square % 8) as u8;
            let to_rank = (to_square / 8) as u8;
            
            let from_notation = format!("{}{}",
                (b'a' + from_file) as char,
                (b'1' + from_rank) as char
            );
            let to_notation = format!("{}{}",
                (b'a' + to_file) as char,
                (b'1' + to_rank) as char
            );

            println!("Engine found move: {} to {}", from_notation, to_notation); // Debug print

            // Find the piece at the source square
            if let Some(piece_index) = self.game.position.pieces.iter().position(|p| {
                bit_scan(p.position) == from_square && p.color == engine_color  // Ensure we're moving the right color
            }) {
                let from_bitboard = self.game.position.pieces[piece_index].position;
                
                // Make the move
                self.game.position.move_piece(from_bitboard, to_square, &game_copy);
                
                println!("Active color after move: {:?}", self.game.position.active_color); // Debug print
                
                // Update evaluation
                let eval = Evaluation::new(self.game.position.clone());
                self.evaluation = eval.evaluate_position();
                
                // Switch turns
                self.is_player_turn = true;
                println!("Engine move complete, switching to player's turn"); // Debug print
            } else {
                println!("No piece found at source square!"); // Debug print
            }
        } else {
            println!("Engine couldn't find a move!"); // Debug print
        }
        
        self.engine_thinking = false;
    }

    fn handle_square_click(&mut self, square: usize) {
        if !self.is_player_turn {
            println!("Not player's turn"); // Debug print
            return;
        }

        // Check if it's the player's color to move
        if self.game.position.active_color != self.player_color {
            println!("Not your color's turn"); // Debug print
            return;
        }

        // Convert the square to internal coordinates if playing as Black
        let internal_square = if self.player_color == Color::Black {
            let rank = square / 8;
            let file = square % 8;
            (7 - rank) * 8 + (7 - file)
        } else {
            square
        };

        if let Some(selected) = self.selected_square {
            // Convert selected square to internal coordinates if playing as Black
            let internal_selected = if self.player_color == Color::Black {
                let rank = selected / 8;
                let file = selected % 8;
                (7 - rank) * 8 + (7 - file)
            } else {
                selected
            };

            // First, find the piece and check if the move is legal
            let piece_index = self.game.position.pieces.iter().position(|p| {
                bit_scan(p.position) == internal_selected && p.color == self.player_color
            });

            if let Some(piece_index) = piece_index {
                let from_bitboard = self.game.position.pieces[piece_index].position;
                
                // Update legal moves
                let game_copy = self.game.clone();
                self.game.position.update_all_legal_moves(&game_copy);
                
                let legal_moves = self.game.position.piece_legal_moves[piece_index];
                
                // Check if the clicked square is a legal destination
                if (legal_moves & (1u64 << internal_square)) != 0 {
                    // Make the move
                    self.game.position.move_piece(from_bitboard, internal_square, &game_copy);
                    
                    // Update evaluation
                    let eval = Evaluation::new(self.game.position.clone());
                    self.evaluation = eval.evaluate_position();
                    
                    // Switch turns and trigger engine move
                    self.is_player_turn = false;
                    println!("Player moved, starting engine move..."); // Debug print
                }
            }
            self.selected_square = None;
        } else {
            // Select the square if it contains a piece of the current player's color
            let has_piece = self.game.position.pieces.iter().any(|p| {
                bit_scan(p.position) == internal_square && p.color == self.player_color
            });
            
            if has_piece {
                self.selected_square = Some(square); // Use display coordinates for highlights
            }
        }
    }

/*     fn draw_evaluation_bar(&self, ui: &mut egui::Ui) {
        let bar_height = ui.available_height() * 0.6;
        let bar_width = 40.0;
        let max_eval = 1000; // Maximum evaluation in centipawns (10 pawns)
        
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Eval");
            ui.add_space(20.0);
            
            let rect = egui::Rect::from_min_size(
                egui::pos2(ui.available_width() / 2.0 - bar_width / 2.0, 60.0),
                egui::vec2(bar_width, bar_height - 80.0),
            );
            
            // Background
            ui.painter().rect_filled(rect, 4.0, egui::Color32::DARK_GRAY);
            
            // Calculate fill height based on evaluation
            let normalized_eval = (self.evaluation.clamp(-max_eval, max_eval) + max_eval) as f32 / (2.0 * max_eval as f32);
            let fill_height = (bar_height - 80.0) * normalized_eval;
            
            // Fill rectangle
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x, rect.max.y - fill_height),
                egui::vec2(bar_width, fill_height),
            );
            
            // Choose color based on who is winning
            let fill_color = if self.evaluation > 0 {
                egui::Color32::from_rgb(100, 200, 100) // Green for white advantage
            } else if self.evaluation < 0 {
                egui::Color32::from_rgb(200, 100, 100) // Red for black advantage
            } else {
                egui::Color32::GRAY // Gray for equal
            };
            
            ui.painter().rect_filled(fill_rect, 4.0, fill_color);
            
            // Draw evaluation text
            let eval_text = format!("{:+.1}", self.evaluation as f32 / 100.0);
            ui.painter().text(
                egui::pos2(rect.center().x, rect.max.y + 20.0),
                egui::Align2::CENTER_TOP,
                eval_text,
                egui::FontId::proportional(16.0),
                egui::Color32::WHITE,
            );
        });
    } */
    fn draw_evaluation_bar(&self, ui: &mut egui::Ui) {
        let bar_height = ui.available_height() * 0.8;
        let bar_width = 20.0;
        let max_eval = 1000; // Maximum evaluation in centipawns (10 pawns)
    
        ui.vertical(|ui| {
            ui.add_space(20.0); // Add padding from top
    
            let rect = egui::Rect::from_min_size(
                egui::pos2(15.0, 59.0), // Align to the left
                egui::vec2(bar_width, bar_height),
            );
    
            // Background
            ui.painter().rect_filled(rect, 4.0, egui::Color32::DARK_GRAY);
    
            let normalized_eval = (self.evaluation.clamp(-max_eval, max_eval) + max_eval) as f32 / (2.0 * max_eval as f32);
            let fill_height = bar_height * normalized_eval;
    
            // Fill rectangle
            let fill_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x, rect.max.y - fill_height),
                egui::vec2(bar_width, fill_height),
            );
    
            // Color based on advantage
            let fill_color = if self.evaluation > 0 {
                egui::Color32::from_rgb(100, 200, 100) // Green for white advantage
            } else if self.evaluation < 0 {
                egui::Color32::from_rgb(200, 100, 100) // Red for black advantage
            } else {
                egui::Color32::GRAY // Gray for equal
            };
    
            ui.painter().rect_filled(fill_rect, 4.0, fill_color);
    
            // Draw evaluation text
            let eval_text = format!("{:+.1}", self.evaluation as f32 / 100.0);
            ui.label(egui::RichText::new(eval_text).size(16.0).strong());
        });
    }
    

    fn draw_board(&mut self, ui: &mut egui::Ui) {
        let board_size = ui.available_width().min(ui.available_height()) - 40.0;
        let square_size = board_size / 8.0;

        // Create a response area for the entire board
        let board_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(board_size, board_size),
        );
        let board_response = ui.allocate_rect(board_rect, egui::Sense::click());

        // Draw the board
        for rank in 0..8 {
            for file in 0..8 {
                // Adjust rank and file based on player color
                let (display_rank, display_file) = if self.player_color == Color::White {
                    (rank, file)
                } else {
                    (7 - rank, 7 - file)
                };

                let square = if self.player_color == Color::White {
                    rank * 8 + file
                } else {
                    (7 - rank) * 8 + (7 - file)
                };

                let is_light = (rank + file) % 2 == 0;
                let rect = egui::Rect::from_min_size(
                    egui::pos2(
                        board_rect.min.x + file as f32 * square_size,
                        board_rect.min.y + (7 - rank) as f32 * square_size,
                    ),
                    egui::vec2(square_size, square_size),
                );

                // Square color
                let color = if Some(square) == self.selected_square {
                    egui::Color32::from_rgb(255, 255, 0) // Bright yellow for selected
                } else if is_light {
                    egui::Color32::from_rgb(240, 217, 181) // Light squares
                } else {
                    egui::Color32::from_rgb(181, 136, 99) // Dark squares
                };

                ui.painter().rect_filled(rect, 0.0, color);

                // Draw piece if present
                if let Some(piece) = self.game.position.pieces.iter().find(|p| {
                    let piece_square = bit_scan(p.position);
                    if self.player_color == Color::White {
                        piece_square == (rank * 8 + file)
                    } else {
                        piece_square == ((7 - rank) * 8 + (7 - file))
                    }
                }) {
                    let piece_char = match (piece.piece_type, piece.color) {
                        (PieceType::Pawn, Color::White) => "♙",
                        (PieceType::Knight, Color::White) => "♘",
                        (PieceType::Bishop, Color::White) => "♗",
                        (PieceType::Rook, Color::White) => "♖",
                        (PieceType::Queen, Color::White) => "♕",
                        (PieceType::King, Color::White) => "♔",
                        (PieceType::Pawn, Color::Black) => "♟",
                        (PieceType::Knight, Color::Black) => "♞",
                        (PieceType::Bishop, Color::Black) => "♝",
                        (PieceType::Rook, Color::Black) => "♜",
                        (PieceType::Queen, Color::Black) => "♛",
                        (PieceType::King, Color::Black) => "♚",
                    };

                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        piece_char,
                        egui::FontId::proportional(square_size * 0.8),
                        if piece.color == Color::White {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        },
                    );
                }

                // Handle clicks
                if board_response.clicked() {
                    if let Some(mouse_pos) = board_response.interact_pointer_pos() {
                        if rect.contains(mouse_pos) {
                            let clicked_square = rank * 8 + file;
                            self.handle_square_click(clicked_square);
                        }
                    }
                }
            }
        }
    }

    fn draw_color_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Choose your color:");
            if ui.button("Play as White").clicked() {
                // Reset everything first
                self.game = Game::new();
                self.player_color = Color::White;
                self.is_player_turn = true;  // White (player) moves first
                self.selected_square = None;
                self.evaluation = 0;
                self.engine_thinking = false;
                
                // Force update of legal moves
                let game_copy = self.game.clone();
                self.game.position.update_all_legal_moves(&game_copy);
                self.game.position.active_color = Color::White;  // Ensure White moves first
                println!("Starting new game - player as White"); // Debug print
            }
            if ui.button("Play as Black").clicked() {
                // Reset everything first
                self.game = Game::new();
                self.player_color = Color::Black;
                self.is_player_turn = false;  // White (engine) moves first
                self.selected_square = None;
                self.evaluation = 0;
                self.engine_thinking = false;
                
                // Force update of legal moves and active color
                let game_copy = self.game.clone();
                self.game.position.update_all_legal_moves(&game_copy);
                self.game.position.active_color = Color::White;  // Ensure White moves first
                println!("Starting new game - player as Black"); // Debug print
                
                // Make first move as White
                self.make_engine_move();
            }
        });
    }
}

impl eframe::App for ChessGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top area for color selection
        egui::containers::panel::TopBottomPanel::top("color_selector").show(ctx, |ui| {
            self.draw_color_selector(ui);
        });

        // Left panel for evaluation bar
        egui::SidePanel::left("eval_bar").min_width(50.0).show(ctx, |ui| {
            self.draw_evaluation_bar(ui);
        });

        // Central panel for the chess board
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_board(ui);
        });

        // If it's the engine's turn, make a move
        if !self.is_player_turn {
            self.make_engine_move();
        }

        // Request continuous redraws
        ctx.request_repaint();
    }
}

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Chess Engine",
        options,
        Box::new(|cc| Box::new(ChessGUI::new(cc)))
    )
}
