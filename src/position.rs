use bitflags::bitflags;
use std::collections::VecDeque;
use crate::utils::*;
use crate::knightattacks::*;
use crate::rayattacks::*;
use crate::movegen_tables::*;
use crate::Game;
use crate::movegeneration::{can_castle, CastlingSide};

type PiecePosition = u64;
type Bitboard = u64;

// File masks for pawn attacks
const FILE_A: u64 = 0x0101010101010101;
const FILE_H: u64 = 0x8080808080808080;

pub fn bit_to_position(bit: PiecePosition) -> Result<String, String> {
    if bit == 0 {
        return Err("No piece present!".to_string());
    } else {
        let onebit_index = bit_scan(bit);
        return Ok(index_to_position(onebit_index));
    }
}

pub fn position_to_bit(position: &str) -> Result<PiecePosition, String> {
    if position.len() != 2 {
        return Err(format!("Invalid length: {}, string: '{}'", position.len(), position));
    }

    let bytes = position.as_bytes();
    let byte0 = bytes[0];
    if byte0 < 97 || byte0 >= 97 + 8 {
        return Err(format!("Invalid column character: {}, string: '{}'", byte0 as char, position));
    }

    let column = (byte0 - 97) as u32;

    let byte1 = bytes[1];
    let row;

    match (byte1 as char).to_digit(10) {
        Some(number) => if number < 1 || number > 8 {
            return Err(format!("Invalid row character: {}, string: '{}'", byte1 as char, position));
        } else {
            row = number - 1;
        },
        None => return Err(format!("Invalid row character: {}, string '{}'", byte1 as char, position)),
    }

    let square_number = row * 8 + column;
    let bit = (1 as u64) << square_number;

    Ok(bit)
}

static COL_MAP: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub fn index_to_position(index: usize) -> String {
    let column = index % 8;
    let row = index / 8 + 1;
    return format!("{}{}", COL_MAP[column], row);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black
}
use Color::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Piece {
    pub position: PiecePosition,
    pub color: Color,
    pub piece_type: PieceType
}

impl Piece {
    fn to_string(&self) -> String {
        let mut result = match self.piece_type {
            PieceType::Pawn => "p ",
            PieceType::Rook => "r ",
            PieceType::Knight => "n ",
            PieceType::Bishop => "b ",
            PieceType::Queen => "q ",
            PieceType::King => "k ",
        }.to_string();

        if self.color == Color::White {
            result.make_ascii_uppercase();
        }

        result
    }
}   

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Square {
    Empty,
    Occupied(usize),
}
use Square::*;

bitflags! {
    pub struct CastlingRights: u8 {
        const NONE = 0;
        const WHITEKINGSIDE = 1 << 0;
        const WHITEQUEENSIDE = 1 << 1;
        const BLACKKINGSIDE = 1 << 2;
        const BLACKQUEENSIDE = 1 << 3;
        const ALL =
            Self::WHITEKINGSIDE.bits
            | Self::WHITEQUEENSIDE.bits
            | Self::BLACKKINGSIDE.bits
            | Self::BLACKQUEENSIDE.bits;
    }
}

/// Represents a complete chess position.
/// 
/// This struct contains all information needed to fully describe a chess position,
/// including piece placement, castling rights, en passant targets, and move counters.
/// It also maintains bitboards for efficient position manipulation and evaluation.
#[derive(Debug, Clone)]
pub struct Position {
    /// Vector of all pieces on the board
    pub pieces: Vec<Piece>,
    /// Vector mapping squares to pieces (Empty or Occupied with piece index)
    pub squares: Vec<Square>,
    /// The color to move next
    pub active_color: Color,
    /// Current castling rights for both colors
    pub castling_rights: CastlingRights,
    /// Square where en passant capture is possible, if any
    pub en_passant: Option<PiecePosition>,
    /// Number of halfmoves since last pawn advance or capture
    pub halfmove_clock: usize,
    /// Number of completed full moves
    pub fullmove_number: usize,
    /// Bitboard of all white pieces
    pub white_occupancy: Bitboard,
    /// Bitboard of all black pieces
    pub black_occupancy: Bitboard,
    /// Whether white kingside castling path is attacked
    pub white_kingside_path_attacked: bool,
    /// Whether white queenside castling path is attacked
    pub white_queenside_path_attacked: bool,
    /// Whether black kingside castling path is attacked
    pub black_kingside_path_attacked: bool,
    /// Whether black queenside castling path is attacked
    pub black_queenside_path_attacked: bool,
    /// Bitboard showing legal moves for each piece
    pub piece_legal_moves: Vec<Bitboard>,
    /// Whether white king has moved from its starting square
    pub white_king_moved: bool,
    /// Whether black king has moved from its starting square
    pub black_king_moved: bool,
    /// Whether white kingside rook has moved from its starting square
    pub white_kingside_rook_moved: bool,
    /// Whether white queenside rook has moved from its starting square
    pub white_queenside_rook_moved: bool,
    /// Whether black kingside rook has moved from its starting square
    pub black_kingside_rook_moved: bool,
    /// Whether black queenside rook has moved from its starting square
    pub black_queenside_rook_moved: bool,
}

impl Position {

    fn push_piece_and_square(&mut self, position: usize, color: Color,
                             piece_type: PieceType, index: &mut usize) {
        self.pieces.push(Piece { position: (1 as u64) << position,
                                 color: color,
                                 piece_type: piece_type });
        self.squares.push(Square::Occupied(*index));

        let bitboard = 1 << position;
        match color {
            Black => self. black_occupancy |= bitboard, 
            White => self.white_occupancy |= bitboard,

        }

        *index += 1;
    }

    fn push_empty_square(&mut self) {
        self.squares.push(Square::Empty);
    }

    pub fn new(game: &Game) -> Position {
        Position::read_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", game)
    }

    pub fn to_string(&self) -> String {
        let mut board = "".to_owned();
        let mut temp = "".to_owned();

        for (i, square) in self.squares.iter().enumerate() {
            match square {
                Square::Empty => temp.push_str(". "),//temp.push_str(&index_to_position(i)),
                Square::Occupied(idx) => temp.push_str(&self.pieces[*idx].to_string()),
            }

            if (i + 1) % 8 == 0 {
                temp.push_str("\n");
                board.insert_str(0, &temp);
                temp.clear();
            }
        }
        board.insert_str(0, &temp);

        board 
    }


    #[allow(non_snake_case)]
    pub fn read_FEN(fen: &str, game: &Game) -> Position {
        let mut Position = Position {
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
        };

        let (position, rest) = split_on(fen, ' ');

        let mut deque_squares = VecDeque::new();
        let mut piece_index = 0;
        let mut piece_position = 64;
        
        for row in position.splitn(8, |ch| ch == '/') {
            piece_position -= 8;
            let (pieces, squares) = parse_row(&row, piece_index, piece_position);
            
            for p in pieces {
                let position = p.position;
                match p.color {
                    Black => Position.black_occupancy |= position,
                    White => Position.white_occupancy |= position,
                }
                Position.pieces.push(p);
                piece_index += 1;
            }
            for s in squares {
                deque_squares.push_front(s);
            }
        }

        Position.squares = Vec::from(deque_squares);


        let (color_to_move, rest) = split_on(rest, ' ');
        Position.active_color = match color_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Unknown color designator: '{}'", color_to_move),
        };


        let (castling_rights, rest) = split_on(rest, ' ');
        let mut castling = CastlingRights::NONE;
        for ch in castling_rights.chars() {
            match ch {
                'K' => castling |= CastlingRights::WHITEKINGSIDE,
                'Q' => castling |= CastlingRights::WHITEQUEENSIDE,
                'k' => castling |= CastlingRights::BLACKKINGSIDE,
                'q' => castling |= CastlingRights::BLACKQUEENSIDE,
                '-' => (),
                other => panic!("Invalid character in castling rights: '{}'", other),
            }
        }
        Position.castling_rights = castling;

        let (en_passant, rest) = split_on(rest, ' ');
        match en_passant {
            "-" => Position.en_passant = None,
            s => match position_to_bit(s) {
                Err(msg) => panic!("{}", msg),
                Ok(bit) => Position.en_passant = Some(bit),
            }
        };


        let (halfmove_clock, rest) = split_on(rest, ' ');
        match halfmove_clock.parse() {
            Ok(number) => Position.halfmove_clock = number,
            Err(_) => panic!("Invalid halfmove: {}", halfmove_clock),
        }

        let (fullmove_number, _rest) = split_on(rest, ' ');
        match fullmove_number.parse() {
            Ok(number) => Position.fullmove_number = number,
            Err(_) => panic!("Invalid halfmove: {}", fullmove_number),
        }

        // Initialize legal moves
        Position.piece_legal_moves.resize(Position.pieces.len(), 0);
        Position.update_all_legal_moves(game);
        Position
    }

    pub fn update_all_legal_moves(&mut self, game: &Game) {
        // Define castling paths
        let white_kingside_path = 0x60;  // f1 and g1
        let white_queenside_path = 0xE;  // b1, c1, and d1
        let black_kingside_path = 0x6000000000000000;  // f8 and g8
        let black_queenside_path = 0xE00000000000000;  // b8, c8, and d8

        // Reset castling path flags
        self.white_kingside_path_attacked = false;
        self.white_queenside_path_attacked = false;
        self.black_kingside_path_attacked = false;
        self.black_queenside_path_attacked = false;

        // Clear and resize the legal moves vector
        self.piece_legal_moves.clear();
        self.piece_legal_moves.resize(self.pieces.len(), 0);

        let all_occupancy = self.white_occupancy | self.black_occupancy;

        // First pass: Calculate legal moves for each piece
        for (i, piece) in self.pieces.iter().enumerate() {
            let square = bit_scan(piece.position);
            let own_occupancy = if piece.color == Color::White { self.white_occupancy } else { self.black_occupancy };
            let opponent_occupancy = if piece.color == Color::White { self.black_occupancy } else { self.white_occupancy };
            
            // Calculate all possible moves for this piece
            let moves = match piece.piece_type {
                PieceType::Pawn => {
                    if piece.color == Color::White {
                        // Forward moves - only if square is empty
                        let one_step = (piece.position << 8) & !all_occupancy;
                        // Double move only allowed from starting rank and if both squares are empty
                        let two_step = if square >= 8 && square < 16 && one_step != 0 {
                            (one_step << 8) & !all_occupancy
                        } else {
                            0
                        };
                        // Diagonal captures - ONLY if there's an opponent piece to capture
                        let diagonal_captures = game.pawn_attacks.white_diagonal_moves[square] & opponent_occupancy;
                        // En passant captures - only if pawn is on rank 5 (squares 32-39)
                        let en_passant_captures = if let Some(ep_square) = self.en_passant {
                            if square >= 32 && square < 40 {  // Only on rank 5
                                game.pawn_attacks.white_diagonal_moves[square] & ep_square
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        // Combine all legal moves
                        one_step | two_step | diagonal_captures | en_passant_captures
                    } else {
                        // Forward moves - only if square is empty
                        let one_step = (piece.position >> 8) & !all_occupancy;
                        // Double move only allowed from starting rank and if both squares are empty
                        let two_step = if square >= 48 && square < 56 && one_step != 0 {
                            (one_step >> 8) & !all_occupancy
                        } else {
                            0
                        };
                        // Diagonal captures - ONLY if there's an opponent piece to capture
                        let diagonal_captures = game.pawn_attacks.black_diagonal_moves[square] & opponent_occupancy;
                        // En passant captures - only if pawn is on rank 4 (squares 24-31)
                        let en_passant_captures = if let Some(ep_square) = self.en_passant {
                            if square >= 24 && square < 32 {  // Only on rank 4
                                game.pawn_attacks.black_diagonal_moves[square] & ep_square
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        // Combine all legal moves
                        one_step | two_step | diagonal_captures | en_passant_captures
                    }
                },
                PieceType::Knight => {
                    let attacks = game.move_gen_tables.knight_attacks[square];
                    // Allow moves to empty squares or squares with opponent pieces
                    attacks & !own_occupancy
                },
                PieceType::Bishop => {
                    let attacks = game.rays.get_bishop_attacks(square, all_occupancy);
                    // Allow moves to empty squares or squares with opponent pieces
                    attacks & !own_occupancy
                },
                PieceType::Rook => {
                    let attacks = game.rays.get_rook_attacks(square, all_occupancy);
                    // Allow moves to empty squares or squares with opponent pieces
                    attacks & !own_occupancy
                },
                PieceType::Queen => {
                    let bishop_attacks = game.rays.get_bishop_attacks(square, all_occupancy);
                    let rook_attacks = game.rays.get_rook_attacks(square, all_occupancy);
                    let attacks = bishop_attacks | rook_attacks;
                    // Allow moves to empty squares or squares with opponent pieces
                    attacks & !own_occupancy
                },
                PieceType::King => {
                    let attacks = game.move_gen_tables.king_attacks[square];
                    // Allow moves to empty squares or squares with opponent pieces
                    attacks & !own_occupancy
                },
            };

            // Store the legal moves for this piece
            self.piece_legal_moves[i] = moves;
        }

        // Second pass: Check attacks on castling paths
        for piece in self.pieces.iter() {
            let square = bit_scan(piece.position);
            
            // Calculate attack squares based on piece type
            let attack_squares = match piece.piece_type {
                PieceType::Pawn => {
                    if piece.color == Color::White {
                        game.pawn_attacks.white_diagonal_moves[square]
                    } else {
                        game.pawn_attacks.black_diagonal_moves[square]
                    }
                },
                PieceType::Knight => game.move_gen_tables.knight_attacks[square],
                PieceType::Bishop => game.rays.get_bishop_attacks(square, all_occupancy),
                PieceType::Rook => game.rays.get_rook_attacks(square, all_occupancy),
                PieceType::Queen => {
                    game.rays.get_bishop_attacks(square, all_occupancy) | 
                    game.rays.get_rook_attacks(square, all_occupancy)
                },
                PieceType::King => game.move_gen_tables.king_attacks[square],
            };

            // White pieces attack black's castling paths
            if piece.color == Color::White {
                if (attack_squares & black_kingside_path) != 0 {
                    self.black_kingside_path_attacked = true;
                }
                if (attack_squares & black_queenside_path) != 0 {
                    self.black_queenside_path_attacked = true;
                }
            }
            // Black pieces attack white's castling paths
            else {
                if (attack_squares & white_kingside_path) != 0 {
                    self.white_kingside_path_attacked = true;
                }
                if (attack_squares & white_queenside_path) != 0 {
                    self.white_queenside_path_attacked = true;
                }
            }
        }
    }

    pub fn move_piece(self: &mut Self, piece_position: Bitboard, new_position: usize, game: &Game) {
        let square_index = bit_scan(piece_position);
        let square = self.squares[square_index];
        let piece_index = match square {
            Empty => panic!("Tried to move piece from an empty square"),
            Occupied(idx) => idx,
        };

        // Check if this is a pawn making a two-square move
        let is_pawn_double_move = {
            let piece = &self.pieces[piece_index];
            if piece.piece_type == PieceType::Pawn {
                let from_rank = square_index / 8;
                let to_rank = new_position / 8;
                if piece.color == Color::White {
                    from_rank == 1 && to_rank == 3  // White pawn moving from rank 2 to 4
                } else {
                    from_rank == 6 && to_rank == 4  // Black pawn moving from rank 7 to 5
                }
            } else {
                false
            }
        };

        // Update piece position
        let new_pos_bit = 1u64 << new_position;
        self.pieces[piece_index].position = new_pos_bit;
        
        // Update occupancy bitboards
        let piece_color = self.pieces[piece_index].color;
        let old_pos_bit = piece_position;
        
        match piece_color {
            Color::White => {
                self.white_occupancy &= !old_pos_bit;
                self.white_occupancy |= new_pos_bit;
            }
            Color::Black => {
                self.black_occupancy &= !old_pos_bit;
                self.black_occupancy |= new_pos_bit;
            }
        }

        // Update squares array
        self.squares[square_index] = Empty;
        self.squares[new_position] = Occupied(piece_index);

        // Set en passant square if this was a pawn double move
        if is_pawn_double_move {
            let ep_square = if piece_color == Color::White {
                1u64 << (new_position - 8)  // One square behind the pawn
            } else {
                1u64 << (new_position + 8)  // One square behind the pawn
            };
            self.en_passant = Some(ep_square);
        } else {
            self.en_passant = None;  // Clear en passant if it wasn't a pawn double move
        }

        // Update castling rights if king or rook moves
        let moving_piece = &self.pieces[piece_index];
        match (moving_piece.piece_type, moving_piece.color) {
            (PieceType::King, Color::White) => {
                self.castling_rights &= !(CastlingRights::WHITEKINGSIDE | CastlingRights::WHITEQUEENSIDE);
                self.white_king_moved = true;
            }
            (PieceType::King, Color::Black) => {
                self.castling_rights &= !(CastlingRights::BLACKKINGSIDE | CastlingRights::BLACKQUEENSIDE);
                self.black_king_moved = true;
            }
            (PieceType::Rook, Color::White) => {
                if square_index == 0 {  // a1
                    self.castling_rights &= !CastlingRights::WHITEQUEENSIDE;
                    self.white_queenside_rook_moved = true;
                } else if square_index == 7 {  // h1
                    self.castling_rights &= !CastlingRights::WHITEKINGSIDE;
                    self.white_kingside_rook_moved = true;
                }
            }
            (PieceType::Rook, Color::Black) => {
                if square_index == 56 {  // a8
                    self.castling_rights &= !CastlingRights::BLACKQUEENSIDE;
                    self.black_queenside_rook_moved = true;
                } else if square_index == 63 {  // h8
                    self.castling_rights &= !CastlingRights::BLACKKINGSIDE;
                    self.black_kingside_rook_moved = true;
                }
            }
            _ => {}
        }

        // Update all legal moves after the move
        self.update_all_legal_moves(game);
    }

    /// Get all legal moves for the current position
    pub fn get_all_legal_moves(&self, game: &Game) -> Vec<u64> {
        let mut moves = Vec::new();
        for (i, legal_moves_bitboard) in self.piece_legal_moves.iter().enumerate() {
            if *legal_moves_bitboard == 0 {
                continue;
            }
            let piece = &self.pieces[i];
            let from_square = bit_scan(piece.position) as u64;
            for to_square in extract_bits(*legal_moves_bitboard) {
                // Encode move: from_square in lower 6 bits, to_square in next 6 bits
                let mut mov = from_square | ((to_square as u64) << 6);
                
                // Set promotion flag for pawns moving to the last rank
                if piece.piece_type == PieceType::Pawn {
                    let to_rank = to_square / 8;
                    if (piece.color == Color::White && to_rank == 7) || 
                       (piece.color == Color::Black && to_rank == 0) {
                        mov |= 1 << 12;  // Set promotion flag
                    }
                }
                moves.push(mov);
            }
        }
        moves
    }

    /// Make a move on the board and return the new position
    pub fn make_move(&mut self, mov: u64) {
        let from_square = mov & 0x3F;
        let to_square = (mov >> 6) & 0x3F;
        let from_bitboard = 1u64 << from_square;
        let to_bitboard = 1u64 << to_square;

        // Find the piece being moved
        if let Some(piece_idx) = self.pieces.iter().position(|p| p.position != 0 && p.position == from_bitboard) {
            // Handle capture if there is one
            if let Square::Occupied(captured_idx) = self.squares[to_square as usize] {
                // Remove the captured piece from the appropriate occupancy bitboard
                match self.pieces[captured_idx].color {
                    Color::White => self.white_occupancy &= !to_bitboard,
                    Color::Black => self.black_occupancy &= !to_bitboard,
                }
                // Mark the captured piece as captured by setting its position to 0
                self.pieces[captured_idx].position = 0;
            }

            // Update piece position
            self.pieces[piece_idx].position = to_bitboard;

            // Update squares
            self.squares[from_square as usize] = Square::Empty;
            self.squares[to_square as usize] = Square::Occupied(piece_idx);

            // Update occupancy bitboards
            match self.pieces[piece_idx].color {
                Color::White => {
                    self.white_occupancy &= !from_bitboard;
                    self.white_occupancy |= to_bitboard;
                },
                Color::Black => {
                    self.black_occupancy &= !from_bitboard;
                    self.black_occupancy |= to_bitboard;
                }
            }

            // Handle promotions
            if mov & (1 << 12) != 0 {
                // Promote to queen
                self.pieces[piece_idx].piece_type = PieceType::Queen;
            }

            // Switch active color
            self.active_color = match self.active_color {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };
        }
    }

    /// Check if the current side to move is in check
    pub fn is_in_check(&self, game: &Game) -> bool {
        // Find the king of the current side
        let king = self.pieces.iter().find(|p| {
            p.piece_type == PieceType::King && p.color == self.active_color
        });

        if let Some(king) = king {
            let king_square = bit_scan(king.position);
            let opponent_color = if self.active_color == Color::White { Color::Black } else { Color::White };
            
            // Check for attacks from opponent's pieces
            for piece in self.pieces.iter().filter(|p| p.color == opponent_color) {
                let piece_square = bit_scan(piece.position);
                let all_occupancy = self.white_occupancy | self.black_occupancy;
                
                // Calculate attack squares based on piece type
                let attacks = match piece.piece_type {
                    PieceType::Pawn => {
                        if piece.color == Color::White {
                            game.pawn_attacks.white_diagonal_moves[piece_square]
                        } else {
                            game.pawn_attacks.black_diagonal_moves[piece_square]
                        }
                    },
                    PieceType::Knight => game.move_gen_tables.knight_attacks[piece_square],
                    PieceType::Bishop => game.rays.get_bishop_attacks(piece_square, all_occupancy),
                    PieceType::Rook => game.rays.get_rook_attacks(piece_square, all_occupancy),
                    PieceType::Queen => {
                        game.rays.get_bishop_attacks(piece_square, all_occupancy) | 
                        game.rays.get_rook_attacks(piece_square, all_occupancy)
                    },
                    PieceType::King => game.move_gen_tables.king_attacks[piece_square],
                };
                
                // If the king's square is in the attack set, it's in check
                if (attacks & king.position) != 0 {
                    return true;
                }
            }
        }
        
        false
    }

    /// Get all capturing moves in the current position
    pub fn get_captures(&self, game: &Game) -> Vec<u64> {
        // TODO: Implement capture move filtering
        // For now, return all moves as a placeholder
        self.get_all_legal_moves(game)
    }

    pub fn get_piece_at(&self, square: u64) -> Option<PieceType> {
        let idx = bit_scan(square);
        match self.squares[idx] {
            Square::Empty => None,
            Square::Occupied(piece_idx) => Some(self.pieces[piece_idx].piece_type),
        }
    }

    pub fn get_piece_type_at(&self, square: u64) -> Option<PieceType> {
        self.pieces.iter()
            .find(|p| p.position == square && p.position != 0)
            .map(|p| p.piece_type)
    }

    pub fn is_capture(&self, mov: u64) -> bool {
        let to_square = 1u64 << ((mov >> 6) & 0x3F);  // Extract to_square from bits 6-11
        self.get_piece_at(to_square).is_some()
    }

    pub fn is_promotion(&self, mov: u64) -> bool {
        mov & (1 << 12) != 0
    }

    pub fn get_hash(&self, game: &Game) -> u64 {
        game.zobrist.hash_position(self)
    }
}

pub fn parse_row(row: &str, mut piece_index: usize, mut piece_position: usize) -> (Vec<Piece>, VecDeque<Square>) {
    let mut pieces = Vec::new();
    let mut squares = VecDeque::new();

    let mut color;


    macro_rules! add_piece {
        ($piece_type:ident) => {
            {
                let piece = Piece {color: color,
                               position: (1 as u64) << piece_position,
                               piece_type: PieceType::$piece_type};
                let square = Square::Occupied(piece_index);
                pieces.push(piece);
                squares.push_front(square);
                piece_position += 1;
                piece_index += 1;
            }
        };
    }

    for ch in row.chars() {
        let is_upper = ch.is_ascii_uppercase();
        color = if is_upper {Color::White} else {Color::Black};
        match ch.to_ascii_lowercase() {
            'r' => add_piece!(Rook),
            'n' => add_piece!(Knight),
            'b' => add_piece!(Bishop),
            'q' => add_piece!(Queen),
            'k' => add_piece!(King),
            'p' => add_piece!(Pawn),
            num => {
                match num.to_digit(10) {
                    None => panic!("Invalid input: {}", num),
                    Some(number) => for i in 0..number {
                        squares.push_front(Square::Empty);
                        piece_position += 1;
                    }
                }
            }
        }
    }

    (pieces, squares)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_initial_position() -> Position {
        let mut Position = Position { pieces: vec![], squares: vec![],
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
        };
        let mut piece_index = 0;

        let color = Color::White;

        Position.push_piece_and_square(0, color,
                                   PieceType::Rook, &mut piece_index);
        Position.push_piece_and_square(1, color,
                                   PieceType::Knight, &mut piece_index);
        Position.push_piece_and_square(2, color,
                                   PieceType::Bishop, &mut piece_index);
        Position.push_piece_and_square(3, color,
                                   PieceType::Queen, &mut piece_index);
        Position.push_piece_and_square(4, color,
                                   PieceType::King, &mut piece_index);
        Position.push_piece_and_square(5, color,
                                   PieceType::Bishop, &mut piece_index);
        Position.push_piece_and_square(6, color,
                                   PieceType::Knight, &mut piece_index);
        Position.push_piece_and_square(7, color,
                                   PieceType::Rook, &mut piece_index);

        for i in 8..16 {
            Position.push_piece_and_square(i, color,
                                       PieceType::Pawn, &mut piece_index);
        }

        for i in 16..48 {
            Position.push_empty_square();
        }

        let color = Color::Black;
        for i in 48..56 {
            Position.push_piece_and_square(i, color,
                                       PieceType::Pawn, &mut piece_index);
        }        

        let offset = 56;
        Position.push_piece_and_square(0 + offset, color,
                                   PieceType::Rook, &mut piece_index);
        Position.push_piece_and_square(1 + offset, color,
                                   PieceType::Knight, &mut piece_index);
        Position.push_piece_and_square(2 + offset, color,
                                   PieceType::Bishop, &mut piece_index);
        Position.push_piece_and_square(3 + offset, color,
                                   PieceType::Queen, &mut piece_index);
        Position.push_piece_and_square(4 + offset, color,
                                   PieceType::King, &mut piece_index);
        Position.push_piece_and_square(5 + offset, color,
                                   PieceType::Bishop, &mut piece_index);
        Position.push_piece_and_square(6 + offset, color,
                                   PieceType::Knight, &mut piece_index);
        Position.push_piece_and_square(7 + offset, color,
                                   PieceType::Rook, &mut piece_index);
                
        
        Position
    }


    #[test]
    fn test_read_fen_initial_position() {
        let game = Game::new();
        let Position = Position::new(&game);
        assert_eq!(Position.active_color, Color::White);
        assert_eq!(Position.castling_rights, CastlingRights::ALL);
        assert_eq!(Position.en_passant, None);
        assert_eq!(Position.halfmove_clock, 0);
        assert_eq!(Position.fullmove_number, 1);
    }

    #[test]
    fn test_read_fen_pieces() {
        let game = Game::new();
        let start = Position::new(&game);
        assert_eq!(start.pieces.len(), 32);
        assert_eq!(start.squares.len(), 64);
    }

    #[test]
    fn test_read_fen_occupancy() {
        let game = Game::new();
        let mut position = Position::new(&game);
        assert_eq!(position.white_occupancy, 0xFFFF);
        assert_eq!(position.black_occupancy, 0xFFFF000000000000);
    }

    #[test]
    fn test_read_fen_black_active() {
        let game = Game::new();
        let Position = Position::read_FEN("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2", &game);
        assert_eq!(Position.active_color, Color::Black);
    }   

    #[test]
    fn test_read_fen_no_castling() {
        let game = Game::new();
        let Position = Position::read_FEN("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2", &game);
        assert_eq!(Position.castling_rights, CastlingRights::NONE);
    }

    #[test]
    fn test_read_fen_en_passant_allowed() {
        let game = Game::new();
        let en_passant_square = "g7";
        let Position = Position::read_FEN(&format!("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq {} 1 2", en_passant_square), &game);
        assert_eq!(Position.en_passant, Some(position_to_bit(en_passant_square).unwrap()));
    }

    #[test]
    fn test_read_fen_moveclocks() {
        let game = Game::new();
        let Position = Position::read_FEN("rnbqkbnr/pp1ppppp/7P/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - g7 1 2", &game);
        assert_eq!(Position.halfmove_clock, 1);
        assert_eq!(Position.fullmove_number, 2);
    }

    #[test]
    fn test_read_fen_castling_rights() {
        let game = Game::new();
        let mut rights = String::new();
        for i in 0..16 {
            if i & 1 != 0 { rights.push('K'); }
            if i & 2 != 0 { rights.push('Q'); }
            if i & 4 != 0 { rights.push('k'); }
            if i & 8 != 0 { rights.push('q'); }
            if rights.is_empty() { rights.push('-'); }

            let mut bitflag_rights = CastlingRights::NONE;
            if i & 1 != 0 { bitflag_rights |= CastlingRights::WHITEKINGSIDE; }
            if i & 2 != 0 { bitflag_rights |= CastlingRights::WHITEQUEENSIDE; }
            if i & 4 != 0 { bitflag_rights |= CastlingRights::BLACKKINGSIDE; }
            if i & 8 != 0 { bitflag_rights |= CastlingRights::BLACKQUEENSIDE; }

            let fen = format!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w {} - 0 1", rights);
            let Position = Position::read_FEN(&fen, &game);
            assert_eq!(Position.castling_rights, bitflag_rights, "FEN: {}\n\n i: {}", fen, i);
            rights.clear();
        }
    }

    #[test]
    fn test_occupancy_start_position() {
        let game = Game::new();
        let start = Position::new(&game);
        let mut white_occupancy = 0;
        for i in 0..16 {
            white_occupancy |= 1 << i;
        }
        let mut black_occupancy = 0;
        for i in 48..64 {
            black_occupancy |= 1 << i;
        }
        assert_eq!(start.white_occupancy, white_occupancy);
        assert_eq!(start.black_occupancy, black_occupancy);
    }

    #[test]
    fn test_move_piece(){
        let game = Game::new();
        let mut position = Position::new(&game);
        position.move_piece(1 << 0, 16, &game);

        assert_eq!(position.pieces[24].position, 1 << 16);
        assert_eq!(position.squares[0], Empty);
        assert_eq!(position.squares[16], Occupied(24))
    }

    #[test]
    fn test_legal_moves_initial_position() {
        let game = Game::new();
        let position = Position::read_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &game);

        // Test black pawns have no diagonal moves initially, but have forward moves
        for i in 8..16 {
            let black_pawn_moves = position.piece_legal_moves[i];
            assert_eq!(black_pawn_moves & position.white_occupancy, 0);
            assert_ne!(black_pawn_moves, 0);
        }

        // Test white knight can move to a3 and c3, but not to squares occupied by own pawns
        let white_knight_1_moves = position.piece_legal_moves[1];  // b1 knight
        let white_knight_2_moves = position.piece_legal_moves[6];  // g1 knight
        assert_ne!(white_knight_1_moves, 0);
    }

    #[test]
    fn test_legal_moves_after_pawn_move() {
        let game = Game::new();
        let position = Position::read_FEN("rnbqkbnr/pp1ppppp/8/2p5/1N2P3/8/PPPP1PPP/R1BQKBNR b KQkq - 1 2", &game);

        // Find the black pawn on c5
        let mut pawn_index = 0;
        for (i, piece) in position.pieces.iter().enumerate() {
            if piece.piece_type == PieceType::Pawn && piece.color == Color::Black {
                let square = bit_scan(piece.position);
                if square == 34 {  // c5 is square 34
                    pawn_index = i;
                    break;
                }
            }
        }

        // Debug prints
        println!("Black pawn position: {:b}", position.pieces[pawn_index].position);
        println!("White occupancy: {:b}", position.white_occupancy);
        println!("Black diagonal moves from c5: {:b}", game.pawn_attacks.black_diagonal_moves[34]);
        println!("Diagonal captures: {:b}", game.pawn_attacks.black_diagonal_moves[34] & position.white_occupancy);
        println!("Expected captures: {:b}", (1u64 << 25) | (1u64 << 27));

        // Test black pawn can capture white knight on b4 and white pawn on d4
        let black_pawn_moves = position.piece_legal_moves[pawn_index];
        assert_ne!(black_pawn_moves & ((1u64 << 25) | (1u64 << 27)), 0);  // b4 and d4 squares
    }

    #[test]
    fn test_legal_moves_multiple_attackers() {
        let game = Game::new();
        // Set up position with white pawns on e4 and g4, white rook on f1, and black pawn on f5
        let position = Position::read_FEN("8/8/8/5p2/4P1P1/8/8/5R2 w - - 0 1", &game);

        // Find the indices of the attacking pieces
        let mut e4_pawn_index = 0;
        let mut g4_pawn_index = 0;
        let mut f1_rook_index = 0;

        for (i, piece) in position.pieces.iter().enumerate() {
            let square = bit_scan(piece.position);
            if piece.piece_type == PieceType::Pawn && square == 28 {  // e4
                e4_pawn_index = i;
            } else if piece.piece_type == PieceType::Pawn && square == 30 {  // g4
                g4_pawn_index = i;
            } else if piece.piece_type == PieceType::Rook && square == 5 {  // f1
                f1_rook_index = i;
            }
        }

        // Get the legal moves for the attacking pieces
        let e4_pawn_moves = position.piece_legal_moves[e4_pawn_index];
        let g4_pawn_moves = position.piece_legal_moves[g4_pawn_index];
        let f1_rook_moves = position.piece_legal_moves[f1_rook_index];

        // Print the moves for debugging
        println!("e4 pawn moves: {}", e4_pawn_moves);
        println!("g4 pawn moves: {}", g4_pawn_moves);
        println!("f1 rook moves: {}", f1_rook_moves);
        println!("f5 square mask: {}", 1u64 << 37);  // f5 is square 37, not 45

        // Check that all three pieces can attack f5 (square 37)
        assert_ne!(e4_pawn_moves & (1u64 << 37), 0);  // e4 pawn can attack f5
        assert_ne!(g4_pawn_moves & (1u64 << 37), 0);  // g4 pawn can attack f5
        assert_ne!(f1_rook_moves & (1u64 << 37), 0);  // f1 rook can attack f5
    }

    #[test]
    fn test_en_passant_capture() {
        let game = Game::new();
        // Set up a position where White has just moved a pawn from e2 to e4,
        // and Black has a pawn on d4 that can capture en passant
        let position = Position::read_FEN("rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", &game);

        // Find the black pawn on d4
        let mut black_pawn_index = 0;
        for (i, piece) in position.pieces.iter().enumerate() {
            if piece.piece_type == PieceType::Pawn && piece.color == Color::Black {
                let square = bit_scan(piece.position);
                if square == 27 {  // d4 is square 27
                    black_pawn_index = i;
                    break;
                }
            }
        }

        // Test that black pawn can capture en passant
        let black_pawn_moves = position.piece_legal_moves[black_pawn_index];
        assert_ne!(black_pawn_moves & (1u64 << 20), 0);  // e3 is square 20
    }

    #[test]
    fn test_castling_flags() {
        let game = Game::new();
        let mut position = Position::read_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &game);

        // Initially, no castling paths should be attacked
        assert!(!position.white_kingside_path_attacked);
        assert!(!position.white_queenside_path_attacked);
        assert!(!position.black_kingside_path_attacked);
        assert!(!position.black_queenside_path_attacked);

        // Move white knight to attack black's kingside castling path
        position.move_piece(1u64 << 1, 18, &game);  // Nb1-c3
        position.move_piece(1u64 << 18, 34, &game);  // Nc3-e4
        position.move_piece(1u64 << 34, 45, &game);  // Ne4-f6 (changed from 50 to 45 for f6)

        // Debug prints
        println!("Knight position: {}", position.pieces[1].position);
        println!("Knight attacks from f6: {:b}", game.move_gen_tables.knight_attacks[45]);
        println!("Black kingside path: {:b}", 0x6000000000000000u64);
        println!("Attack & path: {:b}", game.move_gen_tables.knight_attacks[45] & 0x6000000000000000u64);

        // Black's kingside castling path should now be attacked
        assert!(position.black_kingside_path_attacked);
    }

    #[test]
    fn test_castling_rights() {
        let game = Game::new();
        let mut position = Position::read_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &game);

        // Initially, all castling rights should be available
        assert_eq!(position.castling_rights, CastlingRights::ALL);

        // Move white kingside rook
        position.move_piece(1u64 << 7, 15, &game);  // Rh1-h2

        // White kingside castling should no longer be available
        assert_eq!(position.castling_rights & CastlingRights::WHITEKINGSIDE, CastlingRights::NONE);
    }

    #[test]
    fn test_can_castle_squares_attacked() {
        let game = Game::new();
        let mut position = Position::read_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &game);

        // Initially, castling should not be allowed because the path is blocked
        assert!(!can_castle(&position, Color::White, CastlingSide::Kingside));

        // Move white knight to attack black's kingside castling path
        position.move_piece(1u64 << 1, 18, &game);  // Nb1-c3
        position.move_piece(1u64 << 18, 34, &game);  // Nc3-e4
        position.move_piece(1u64 << 34, 50, &game);  // Ne4-f6

        // Castling should still not be allowed because the bishop is still blocking the path
        assert!(!can_castle(&position, Color::White, CastlingSide::Kingside));
    }
}
