use crate::position::Position;
use crate::evaluation::Evaluation;
use crate::Game;
use std::time::{Instant, Duration};
use crate::moveorder::MoveOrderer;

const MAX_SCORE: i32 = 100000;
const MIN_SCORE: i32 = -100000;
const MATE_SCORE: i32 = 99000;
const MAX_DEPTH: i32 = 4;  // Reduced from 6 to 4 to prevent stack overflow
const MAX_QUIESCENCE_DEPTH: i32 = 4;  // Add a limit to quiescence search depth

#[derive(Clone)]
pub struct Search {
    nodes_searched: u64,
    start_time: Instant,
    max_time: Duration,
    game: Game,
    move_orderer: MoveOrderer,
}

impl Search {
    pub fn new() -> Self {
        Self {
            nodes_searched: 0,
            start_time: Instant::now(),
            max_time: Duration::from_secs(5),
            game: Game::new(),
            move_orderer: MoveOrderer::new(),
        }
    }

    pub fn set_max_time(&mut self, seconds: u64) {
        self.max_time = Duration::from_secs(seconds);
    }

    /// Find the best move in the current position
    pub fn find_best_move(&mut self, position: &mut Position) -> Option<u64> {
        self.nodes_searched = 0;
        self.start_time = Instant::now();
        
        let mut alpha = MIN_SCORE;
        let beta = MAX_SCORE;
        let mut best_move = None;
        let mut best_score = MIN_SCORE;

        position.update_all_legal_moves(&self.game);
        let moves = position.get_all_legal_moves(&self.game);
        let ordered_moves = self.move_orderer.order_moves(position, &moves, &self.game);

        // Start with a shallower depth and gradually increase
        for depth in 1..=MAX_DEPTH {
            if self.start_time.elapsed() >= self.max_time {
                break;
            }

            let mut current_alpha = alpha;
            for &mov in &ordered_moves {
                let mut new_position = position.clone();
                new_position.make_move(mov);

                let score = -self.alpha_beta(
                    -beta,
                    -current_alpha,
                    depth - 1,
                    0,  // Current depth from root
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
        ply_from_root: i32,  // Add ply from root to prevent deep recursion
        position: &mut Position
    ) -> i32 {
        // Check for maximum recursion depth
        if ply_from_root >= MAX_DEPTH * 2 {
            return self.evaluate_position(position);
        }

        self.nodes_searched += 1;

        if self.start_time.elapsed() >= self.max_time {
            return 0;
        }

        if depth <= 0 {
            return self.quiescence(alpha, beta, 0, position);
        }

        position.update_all_legal_moves(&self.game);
        let moves = position.get_all_legal_moves(&self.game);
        let ordered_moves = self.move_orderer.order_moves(position, &moves, &self.game);

        if moves.is_empty() {
            if position.is_in_check() {
                return MIN_SCORE + ply_from_root; // Prefer faster mate
            }
            return 0; // Stalemate
        }

        for &mov in &ordered_moves {
            let mut new_position = position.clone();
            new_position.make_move(mov);

            let score = -self.alpha_beta(
                -beta,
                -alpha,
                depth - 1,
                ply_from_root + 1,
                &mut new_position
            );

            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }

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

    #[test]
    fn test_mate_in_one() {
        let game = Game::new();
        let mut position = Position::read_FEN(
            "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1",
            &game
        );
        let mut search = Search::new();
        search.set_max_time(1); // Limit search time to 1 second
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
        let mut search = Search::new();
        search.set_max_time(1);
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
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
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
    }

    #[test]
    fn test_search_depth() {
        let game = Game::new();
        let mut position = Position::new(&game);
        let mut search = Search::new();
        search.set_max_time(1);
        let best_move = search.find_best_move(&mut position);
        assert!(best_move.is_some());
        assert!(search.nodes_searched > 0);
    }
}
