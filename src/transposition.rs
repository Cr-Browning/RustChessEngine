pub struct TranspositionEntry {
    hash: u64,
    depth: i32,
    flag: NodeType,
    value: i32,
    best_move: Option<Move>,
}

pub struct TranspositionTable {
    table: Vec<Option<TranspositionEntry>>,
    size: usize,
}
