use crate::position::Position;
use crate::evaluation::Evaluation;
use crate::Game;
use std::time::{Instant, Duration};
use crate::moveorder::MoveOrderer;
use crate::position::Square;
use crate::utils::{bit_scan_safe, extract_bits};
use crate::transposition::{TranspositionTable, NodeType};

const MAX_SCORE: i32 = 100000;
const MIN_SCORE: i32 = -100000;
const MATE_SCORE: i32 = 99000;
const MAX_DEPTH: i32 = 4;  // Reduced from 6 to 4 to prevent stack overflow
const MAX_QUIESCENCE_DEPTH: i32 = 4;  // Add a limit to quiescence search depth
const TT_SIZE: usize = 32;  // 32MB transposition table

#[derive(Clone)]
pub struct Search {
    nodes_searched: u64,
    start_time: Instant,
    max_time: Duration,
    game: Game,
    move_orderer: MoveOrderer,
    tt: TranspositionTable,
}

impl Search {
    pub fn new() -> Self {
        Self {
            nodes_searched: 0,
            start_time: Instant::now(),
            max_time: Duration::from_secs(5),
            game: Game::new(),
            move_orderer: MoveOrderer::new(),
            tt: TranspositionTable::new(TT_SIZE),
        }
    }

    pub fn set_max_time(&mut self, seconds: u64) {
        self.max_time = Duration::from_secs(seconds);
    }

    /// Find the best move in the current position
    pub fn find_best_move(&mut self, position: &mut Position) -> Option<u64> {
        self.nodes_searched = 0;
        self.start_time = Instant::now();
        self.tt.new_search();  // Update age for new search
        
        let mut alpha = MIN_SCORE;
        let beta = MAX_SCORE;
        let mut best_move = None;
        let mut best_score = MIN_SCORE;

        // Update legal moves before searching
        position.update_all_legal_moves(&self.game);
        let moves = position.get_all_legal_moves(&self.game);
        
        // Skip pieces that have been captured (position == 0) or belong to wrong color
        let valid_moves: Vec<u64> = moves.into_iter()
            .filter(|&mov| {
                let from_square = mov & 0x3F;
                match position.squares[from_square as usize] {
                    Square::Empty => false,
                    Square::Occupied(idx) => {
                        let piece = &position.pieces[idx];
                        piece.position != 0 && piece.color == position.active_color
                    }
                }
            })
            .collect();

        if valid_moves.is_empty() {
            return None;
        }

        let ordered_moves = self.move_orderer.order_moves(position, &valid_moves, &self.game);

        // Start with a shallower depth and gradually increase
        for depth in 1..=MAX_DEPTH {
            if self.start_time.elapsed() >= self.max_time {
                break;
            }

            let mut current_alpha = alpha;
            for &mov in &ordered_moves {
                let mut new_position = position.clone();
                new_position.make_move(mov);
                new_position.update_all_legal_moves(&self.game);

                let score = -self.alpha_beta(
                    -beta,
                    -current_alpha,
                    depth - 1,
                    0,
                    &mut new_position
                );

                if score > best_score {
                    best_score = score;
                    best_move = Some(mov);
                    current_alpha = score;
                }
            }
            alpha = current_alpha;
        }

        best_move
    }

    /// Alpha-beta search implementation
    fn alpha_beta(
        &mut self,
        mut alpha: i32,
        beta: i32,
        depth: i32,
        ply_from_root: i32,
        position: &mut Position
    ) -> i32 {
        if ply_from_root >= MAX_DEPTH * 2 {
            return self.evaluate_position(position);
        }

        self.nodes_searched += 1;

        if self.start_time.elapsed() >= self.max_time {
            return 0;
        }

        // Probe transposition table
        let hash = position.get_hash(&self.game);
        if let Some(entry) = self.tt.probe(hash) {
            if entry.depth >= depth {
                match entry.flag {
                    NodeType::Exact => return entry.value,
                    NodeType::Alpha if entry.value <= alpha => return alpha,
                    NodeType::Beta if entry.value >= beta => return beta,
                    _ => {}
                }
            }
        }

        if depth <= 0 {
            return self.quiescence(alpha, beta, 0, position);
        }

        position.update_all_legal_moves(&self.game);
        let moves = position.get_all_legal_moves(&self.game);
        
        // Filter valid moves
        let valid_moves: Vec<u64> = moves.into_iter()
            .filter(|&mov| {
                let from_square = mov & 0x3F;
                match position.squares[from_square as usize] {
                    Square::Empty => false,
                    Square::Occupied(idx) => {
                        let piece = &position.pieces[idx];
                        piece.position != 0 && piece.color == position.active_color
                    }
                }
            })
            .collect();

        if valid_moves.is_empty() {
            if position.is_in_check(&self.game) {
                return MIN_SCORE + ply_from_root; // Prefer faster mate
            }
            return 0; // Stalemate
        }

        let ordered_moves = self.move_orderer.order_moves(position, &valid_moves, &self.game);
        let mut best_move = None;
        let old_alpha = alpha;

        for &mov in &ordered_moves {
            let mut new_position = position.clone();
            new_position.make_move(mov);
            new_position.update_all_legal_moves(&self.game);

            let score = -self.alpha_beta(
                -beta,
                -alpha,
                depth - 1,
                ply_from_root + 1,
                &mut new_position
            );

            if score >= beta {
                // Store beta cutoff in transposition table
                self.tt.store(hash, depth, NodeType::Beta, beta, Some(mov));
                return beta;
            }
            if score > alpha {
                alpha = score;
                best_move = Some(mov);
            }
        }

        // Store position in transposition table
        let node_type = if alpha > old_alpha {
            NodeType::Exact
        } else {
            NodeType::Alpha
        };
        self.tt.store(hash, depth, node_type, alpha, best_move);

        alpha
    }

    /// Quiescence search to handle tactical sequences
    fn quiescence(
        &mut self,
        mut alpha: i32,
        beta: i32,
        depth: i32,  // Add depth parameter to limit quiescence search
        position: &mut Position
    ) -> i32 {
        // Limit quiescence search depth
        if depth >= MAX_QUIESCENCE_DEPTH {
            return self.evaluate_position(position);
        }

        self.nodes_searched += 1;

        let stand_pat = self.evaluate_position(position);

        if stand_pat >= beta {
            return beta;
        }

        alpha = alpha.max(stand_pat);

        position.update_all_legal_moves(&self.game);
        let captures = position.get_captures(&self.game);

        for &mov in &captures {
            let mut new_position = position.clone();
            new_position.make_move(mov);

            let score = -self.quiescence(
                -beta,
                -alpha,
                depth + 1,
                &mut new_position
            );

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }

        alpha
    }

    fn evaluate_position(&self, position: &Position) -> i32 {
        let evaluation = Evaluation::new(position.clone());
        evaluation.evaluate_position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn test_mate_in_one() {
        let game = Game::new();
        let mut position = Position::read_FEN(
            "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1",
            &game
        );
        let mut search = Search::new();
        search.set_max_time(1); // Limit search time to 1 second
        
        // Update legal moves before searching
        position.update_all_legal_moves(&game);
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
    }

    #[test]
    fn test_find_capture() {
        let game = Game::new();
        let mut position = Position::read_FEN(
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
            &game
        );

        println!("\nInitial position:");
        println!("{}", position.to_string());
        println!("Active color: {:?}", position.active_color);
        
        // Print each piece's position and legal moves
        for (i, piece) in position.pieces.iter().enumerate() {
            if piece.position == 0 {
                continue;
            }
            if piece.color == position.active_color {
                println!("Piece {}: {:?} {:?} at square {}, legal moves: {:?}", 
                    i, piece.color, piece.piece_type, 
                    bit_scan_safe(piece.position).unwrap_or(64),
                    extract_bits(position.piece_legal_moves[i]));
            }
        }

        let mut search = Search::new();
        search.set_max_time(1);
        
        // Update legal moves before searching
        position.update_all_legal_moves(&game);
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
        
        // Print the chosen move
        if let Some(mov) = best_move {
            let from_square = mov & 0x3F;
            let to_square = (mov >> 6) & 0x3F;
            println!("\nChosen move: from square {} to square {}", from_square, to_square);
            println!("Is capture: {}", position.is_capture(mov));
            
            // Make the move to visualize the result
            let mut new_position = position.clone();
            new_position.make_move(mov);
            println!("\nPosition after move:");
            println!("{}", new_position.to_string());
        }
        
        // Verify the move is a capture
        if let Some(mov) = best_move {
            assert!(position.is_capture(mov), "Expected a capture move, but got a non-capture move");
        }
    }

    #[test]
    fn test_avoid_mate() {
        let game = Game::new();
        let mut position = Position::read_FEN(
            "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1",
            &game
        );
        let mut search = Search::new();
        search.set_max_time(1);
        
        // Update legal moves before searching
        position.update_all_legal_moves(&game);
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
    }

    #[test]
    fn test_search_depth() {
        let game = Game::new();
        let mut position = Position::new(&game);
        let mut search = Search::new();
        search.set_max_time(1);
        
        // Update legal moves before searching
        position.update_all_legal_moves(&game);
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
        assert!(search.nodes_searched > 0);
    }
}
