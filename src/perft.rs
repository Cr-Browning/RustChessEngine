use crate::position::Position;
    
pub struct Perft {
    nodes: u64,
    captures: u64,
    en_passants: u64,
    castles: u64,
    promotions: u64,
}

impl Perft {
    pub fn new() -> Self {
        Perft {
            nodes: 0,
            captures: 0,
            en_passants: 0,
            castles: 0,
            promotions: 0,
        }
    }

    pub fn run(&mut self, position: &Position, depth: i32) -> u64 {
        // Performance test implementation
        self.nodes
    }
}
