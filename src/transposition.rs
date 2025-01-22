use crate::chess_move::Move;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NodeType {
    Exact,    // Exact score
    Alpha,    // Upper bound
    Beta,     // Lower bound
}

#[derive(Copy, Clone)]
pub struct TranspositionEntry {
    pub hash: u64,         // Zobrist hash of position
    pub depth: i32,        // Depth searched
    pub flag: NodeType,    // Type of node
    pub value: i32,        // Score of position
    pub best_move: Option<u64>, // Best move found (encoded as u64)
    pub age: u8,          // Age for replacement strategy
}

#[derive(Clone)]
pub struct TranspositionTable {
    table: Vec<Option<TranspositionEntry>>,
    size: usize,
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        // Calculate number of entries that fit in size_mb megabytes
        let entry_size = std::mem::size_of::<TranspositionEntry>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        
        TranspositionTable {
            table: vec![None; num_entries],
            size: num_entries,
            age: 0,
        }
    }

    pub fn store(&mut self, hash: u64, depth: i32, flag: NodeType, value: i32, best_move: Option<u64>) {
        let index = self.get_index(hash);
        let entry = TranspositionEntry {
            hash,
            depth,
            flag,
            value,
            best_move,
            age: self.age,
        };

        // Replacement strategy: always replace if deeper search or older age
        if let Some(existing) = self.table[index] {
            if existing.depth <= depth || existing.age != self.age {
                self.table[index] = Some(entry);
            }
        } else {
            self.table[index] = Some(entry);
        }
    }

    pub fn probe(&self, hash: u64) -> Option<&TranspositionEntry> {
        let index = self.get_index(hash);
        if let Some(entry) = &self.table[index] {
            if entry.hash == hash {
                return Some(entry);
            }
        }
        None
    }

    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        self.table.fill(None);
        self.age = 0;
    }

    fn get_index(&self, hash: u64) -> usize {
        (hash as usize) % self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_probe() {
        let mut tt = TranspositionTable::new(1); // 1MB table
        let hash = 123456789;
        let depth = 4;
        let flag = NodeType::Exact;
        let value = 100;
        let best_move = Some(0x1234u64); // Example encoded move

        tt.store(hash, depth, flag, value, best_move);
        let entry = tt.probe(hash).unwrap();

        assert_eq!(entry.hash, hash);
        assert_eq!(entry.depth, depth);
        assert_eq!(entry.flag, flag);
        assert_eq!(entry.value, value);
        assert_eq!(entry.best_move, best_move);
    }

    #[test]
    fn test_replacement_strategy() {
        let mut tt = TranspositionTable::new(1);
        let hash = 123456789;

        // Store initial entry
        tt.store(hash, 2, NodeType::Exact, 100, Some(0x1234u64));

        // Store deeper search entry
        tt.store(hash, 4, NodeType::Exact, 200, Some(0x5678u64));
        let entry = tt.probe(hash).unwrap();
        assert_eq!(entry.depth, 4);
        assert_eq!(entry.value, 200);
        assert_eq!(entry.best_move, Some(0x5678u64));

        // Try to store shallower search entry
        tt.store(hash, 1, NodeType::Exact, 300, Some(0x9ABCu64));
        let entry = tt.probe(hash).unwrap();
        assert_eq!(entry.depth, 4); // Should keep deeper entry
        assert_eq!(entry.value, 200);
        assert_eq!(entry.best_move, Some(0x5678u64));
    }

    #[test]
    fn test_age_update() {
        let mut tt = TranspositionTable::new(1);
        let hash = 123456789;

        tt.store(hash, 4, NodeType::Exact, 100, Some(0x1234u64));
        let initial_age = tt.probe(hash).unwrap().age;

        tt.new_search();
        tt.store(hash, 3, NodeType::Exact, 200, Some(0x5678u64));
        let new_age = tt.probe(hash).unwrap().age;

        assert_ne!(initial_age, new_age);
    }
}
