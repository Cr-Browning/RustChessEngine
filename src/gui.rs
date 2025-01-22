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
    move_history: Vec<String>,  // Add move history
    dragging_piece: Option<(usize, egui::Pos2)>,  // Add drag and drop support
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
            move_history: Vec::new(),
            dragging_piece: None,
        }
    }

    fn format_move(&self, from: usize, to: usize, piece_type: PieceType) -> String {
        let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let piece_symbol = match piece_type {
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Rook => "R",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Pawn => "",
        };
        
        let from_file = files[from % 8];
        let from_rank = (from / 8) + 1;
        let to_file = files[to % 8];
        let to_rank = (to / 8) + 1;
        
        format!("{}{}{}{}{}", piece_symbol, from_file, from_rank, to_file, to_rank)
    }

    fn make_engine_move(&mut self) {
        if self.is_player_turn || self.engine_thinking {
            return;
        }

        // Verify it's actually the engine's turn based on colors
        if (self.player_color == Color::White && self.game.position.active_color == Color::White) ||
           (self.player_color == Color::Black && self.game.position.active_color == Color::Black) {
            return;
        }

        self.engine_thinking = true;

        // Update legal moves before searching
        let game_copy = self.game.clone();
        self.game.position.update_all_legal_moves(&game_copy);

        // Check for checkmate/stalemate
        if self.game.position.get_all_legal_moves(&game_copy).is_empty() {
            if self.game.position.is_in_check(&game_copy) {
                println!("Checkmate! Player wins!");
            } else {
                println!("Stalemate! Game is drawn.");
            }
            self.engine_thinking = false;
            return;
        }

        // Find best move using alpha-beta search
        let mut position_copy = self.game.position.clone();
        if let Some(best_move) = self.search.find_best_move(&mut position_copy) {
            let from_square = (best_move & 0x3F) as usize;
            let to_square = ((best_move >> 6) & 0x3F) as usize;
            
            // Get piece type for move notation
            let piece_type = self.game.position.pieces.iter()
                .find(|p| bit_scan(p.position) == from_square)
                .map(|p| p.piece_type)
                .unwrap_or(PieceType::Pawn);
            
            // Make the move
            self.game.position.make_move(best_move);
            
            // Add to move history
            let move_text = self.format_move(from_square, to_square, piece_type);
            self.move_history.push(format!("{}. ... {}", self.move_history.len() / 2 + 1, move_text));
            
            // Update evaluation
            let eval = Evaluation::new(self.game.position.clone());
            self.evaluation = eval.evaluate_position();
            
            self.is_player_turn = true;
        }
        
        self.engine_thinking = false;
    }

    fn handle_square_click(&mut self, square: usize, pointer_pos: Option<egui::Pos2>) {
        // Validate square is in bounds
        if square >= 64 {
            return;
        }

        if !self.is_player_turn {
            return;
        }

        // Verify it's the player's turn based on colors
        if (self.player_color == Color::White && self.game.position.active_color == Color::Black) ||
           (self.player_color == Color::Black && self.game.position.active_color == Color::White) {
            return;
        }

        let internal_square = if self.player_color == Color::Black {
            let rank = 7 - (square / 8);
            let file = 7 - (square % 8);
            rank * 8 + file
        } else {
            square
        };

        if let Some(pos) = pointer_pos {
            // Start dragging
            let has_piece = self.game.position.pieces.iter().any(|p| {
                bit_scan(p.position) == internal_square && p.color == self.player_color
            });
            
            if has_piece {
                self.dragging_piece = Some((square, pos));
                self.selected_square = Some(square);
            }
            return;
        }

        // Handle piece drop or regular click
        if let Some((selected, _)) = self.dragging_piece.take() {
            if selected != square {  // Only make a move if the destination is different
                self.handle_move(selected, square);
            }
            self.selected_square = None;
        } else if let Some(selected) = self.selected_square {
            if selected != square {  // Only make a move if the destination is different
                self.handle_move(selected, square);
            }
            self.selected_square = None;
        } else {
            // Select the square if it contains a piece of the current player's color
            let has_piece = self.game.position.pieces.iter().any(|p| {
                bit_scan(p.position) == internal_square && p.color == self.player_color
            });
            
            if has_piece {
                self.selected_square = Some(square);
            }
        }
    }

    fn handle_move(&mut self, from_square: usize, to_square: usize) {
        // Validate squares are in bounds
        if from_square >= 64 || to_square >= 64 {
            return;
        }

        let internal_from = if self.player_color == Color::Black {
            let rank = 7 - (from_square / 8);
            let file = 7 - (from_square % 8);
            rank * 8 + file
        } else {
            from_square
        };

        let internal_to = if self.player_color == Color::Black {
            let rank = 7 - (to_square / 8);
            let file = 7 - (to_square % 8);
            rank * 8 + file
        } else {
            to_square
        };

        let piece_index = self.game.position.pieces.iter().position(|p| {
            bit_scan(p.position) == internal_from && p.color == self.player_color
        });

        if let Some(piece_index) = piece_index {
            let game_copy = self.game.clone();
            self.game.position.update_all_legal_moves(&game_copy);
            
            let legal_moves = self.game.position.piece_legal_moves[piece_index];
            
            if (legal_moves & (1u64 << internal_to)) != 0 {
                let mov = internal_from as u64 | ((internal_to as u64) << 6);
                
                // Get piece type for move notation
                let piece_type = self.game.position.pieces[piece_index].piece_type;
                
                // Make the move
                self.game.position.make_move(mov);
                
                // Add to move history
                let move_text = self.format_move(internal_from, internal_to, piece_type);
                if self.player_color == Color::White {
                    self.move_history.push(format!("{}. {}", self.move_history.len() / 2 + 1, move_text));
                } else {
                    self.move_history.push(format!("{}. ... {}", self.move_history.len() / 2 + 1, move_text));
                }
                
                // Update evaluation
                let eval = Evaluation::new(self.game.position.clone());
                self.evaluation = eval.evaluate_position();
                
                // Check for game end conditions
                self.game.position.update_all_legal_moves(&game_copy);
                if self.game.position.get_all_legal_moves(&game_copy).is_empty() {
                    if self.game.position.is_in_check(&game_copy) {
                        println!("Checkmate! Player wins!");
                    } else {
                        println!("Stalemate! Game is drawn.");
                    }
                } else {
                    // Switch turns only if the move was successful
                    self.is_player_turn = false;
                }
            }
        }
    }

    fn draw_evaluation_bar(&self, ui: &mut egui::Ui) {
        let bar_height = ui.available_height() * 0.8;
        let bar_width = 20.0;
        let max_eval = 1000; // Maximum evaluation in centipawns (10 pawns)
    
        ui.vertical(|ui| {
            ui.add_space(20.0); // Add padding from top
    
            let rect = egui::Rect::from_min_size(
                egui::pos2(ui.available_width() / 2.0 - bar_width / 2.0, 60.0), // Center horizontally
                egui::vec2(bar_width, bar_height - 20.0), // Adjust height for better proportions
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

        let board_rect = egui::Rect::from_min_size(
            ui.cursor().min,
            egui::vec2(board_size, board_size),
        );
        let board_response = ui.allocate_rect(board_rect, egui::Sense::click_and_drag());

        // Handle mouse interactions
        if let Some(pointer_pos) = board_response.hover_pos() {
            let file = ((pointer_pos.x - board_rect.min.x) / square_size).floor() as isize;
            let rank = 7 - ((pointer_pos.y - board_rect.min.y) / square_size).floor() as isize;
            
            if file >= 0 && file < 8 && rank >= 0 && rank < 8 {
                let square = (rank * 8 + file) as usize;
                
                if board_response.clicked() {
                    self.handle_square_click(square, Some(pointer_pos));
                } else if board_response.drag_released() {
                    self.handle_square_click(square, None);
                }
            }
        }

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

                // Check if this square contains a king in check/checkmate
                let mut is_check = false;
                let mut is_checkmate = false;
                if let Some(piece) = self.game.position.pieces.iter().find(|p| {
                    let piece_square = bit_scan(p.position);
                    if self.player_color == Color::White {
                        piece_square == (rank * 8 + file)
                    } else {
                        piece_square == ((7 - rank) * 8 + (7 - file))
                    }
                }) {
                    if piece.piece_type == PieceType::King && piece.color == self.game.position.active_color {
                        is_check = self.game.position.is_in_check(&self.game);
                        if is_check {
                            let legal_moves = self.game.position.get_all_legal_moves(&self.game);
                            is_checkmate = legal_moves.is_empty();
                        }
                    }
                }

                // Draw square with appropriate color
                let final_color = if is_checkmate {
                    egui::Color32::from_rgb(255, 0, 0) // Red for checkmate
                } else if is_check {
                    egui::Color32::from_rgb(255, 255, 0) // Yellow for check
                } else {
                    color
                };

                ui.painter().rect_filled(rect, 0.0, final_color);

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
            }
        }

        // Draw dragged piece if any
        if let Some((square, pos)) = self.dragging_piece {
            if let Some(piece) = self.game.position.pieces.iter().find(|p| {
                let piece_square = bit_scan(p.position);
                if self.player_color == Color::White {
                    piece_square == square
                } else {
                    piece_square == ((7 - square / 8) * 8 + (7 - square % 8))
                }
            }) {
                // Draw piece at cursor position
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
                    pos,
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

    // Add a function to draw the move list
    fn draw_move_list(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Move History");
            ui.add_space(10.0);
            
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 60.0)
                .show(ui, |ui| {
                    for move_text in &self.move_history {
                        ui.label(move_text);
                    }
                });
        });
    }
}

impl eframe::App for ChessGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark mode
        ctx.set_visuals(egui::Visuals::dark());

        // Top panel for title and color selection
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading("RustChess Engine");
                ui.add_space(20.0);
                if ui.button("Play as White").clicked() {
                    self.game = Game::new();
                    self.player_color = Color::White;
                    self.is_player_turn = true;
                    self.selected_square = None;
                    self.evaluation = 0;
                    self.engine_thinking = false;
                    let game_copy = self.game.clone();
                    self.game.position.update_all_legal_moves(&game_copy);
                    self.game.position.active_color = Color::White;
                }
                if ui.button("Play as Black").clicked() {
                    self.game = Game::new();
                    self.player_color = Color::Black;
                    self.is_player_turn = false;
                    self.selected_square = None;
                    self.evaluation = 0;
                    self.engine_thinking = false;
                    let game_copy = self.game.clone();
                    self.game.position.update_all_legal_moves(&game_copy);
                    self.game.position.active_color = Color::White;
                    self.make_engine_move();
                }
            });
            ui.add_space(10.0);
        });

        // Left panel for evaluation bar
        egui::SidePanel::left("eval_panel")
            .exact_width(60.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.heading("Eval");
                    ui.add_space(10.0);
                    self.draw_evaluation_bar(ui);
                });
            });

        // Right panel for move history (placeholder for future implementation)
        egui::SidePanel::right("moves_panel")
            .exact_width(200.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.draw_move_list(ui);
            });

        // Central panel for the chess board
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                self.draw_board(ui);
                ui.add_space(20.0);
            });
        });

        // Bottom panel for status messages
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(if self.is_player_turn {
                    "Your turn to move"
                } else {
                    "Engine is thinking..."
                });
                if self.game.position.is_in_check(&self.game) {
                    ui.label("CHECK!");
                }
            });
            ui.add_space(10.0);
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
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "RustChess Engine",
        options,
        Box::new(|cc| Box::new(ChessGUI::new(cc)))
    )
}
