// use rand::{rngs::StdRng, Rng, SeedableRng};

// use crate::position::Position;

// #[derive(Clone, Copy, PartialEq)]
// pub enum Bound {
//     Exact,
//     Lower,
//     Upper
// }

// #[derive(Clone, Copy)]
// struct TranspositionEntry {
//     hash: u64,
//     generation: u32,
//     depth: i32,
//     score: f32,
//     bound: Bound
// }

// pub struct TranspositionTable {
//     pub used_entires: usize,
//     generation: u32,
//     hash_bits: Vec<u64>,
//     entires: Vec<Option<TranspositionEntry>>
// }

// impl TranspositionTable {
//     pub fn new(size: usize) -> Self {
//         let mut hash_bits: Vec<u64> = Vec::with_capacity(64 * 12 + 1);
//         let mut rng = StdRng::seed_from_u64(65842);
//         for i in 0..(64 * 12 + 1) {
//             hash_bits.push(rng.random());
//         }
//         Self {
//             used_entires: 0,
//             generation: 0,
//             entires: vec![None; size],
//             hash_bits
//         }
//     }

//     pub fn reset(&mut self) {
//         if self.used_entires == 0 {
//             return;
//         }
//         self.generation += 1;
//         self.used_entires = 0;
//     }

//     pub fn hash(&self, pos: &Position) -> u64 {
//         let pieces = [pos.pawns, pos.knights, pos.bishops, pos.rooks, pos.queens, pos.kings];
//         let mut hash: u64 = 0;
//         let mut index = 0;
//         for piece in pieces {
//             for s in piece.intersect(pos.white_pieces) {
//                 hash ^= self.hash_bits[index + s.rank as usize * 8 + s.file as usize];
//             }
//             index += 64;
//             for s in piece.intersect(pos.black_pieces) {
//                 hash ^= self.hash_bits[index + s.rank as usize * 8 + s.file as usize];
//             }
//             index += 64;
//         }
//         if pos.white_to_play {
//             hash ^= self.hash_bits[64 * 12];
//         }
//         return hash;
//     }

//     pub fn size(&self) -> usize {
//         return self.entires.len();
//     }

//     pub fn get_score(&self, pos: &Position, depth: i32) -> Option<(f32, Bound)> {
//         let hash = self.hash(pos);
//         match self.entires[(hash as usize) % self.entires.len()] {
//             Some(e) => {
//                 if e.hash == hash && e.generation == self.generation && e.depth >= depth {
//                     Some((e.score, e.bound))
//                 } else {
//                     None
//                 }
//             }
//             None => None
//         }
//     }

//     pub fn set_score(&mut self, pos: &Position, depth: i32, score: f32, bound: Bound) {
//         let hash = self.hash(pos);
//         let index = (hash as usize) % self.entires.len();
//         if let Some(e) = self.entires[index] {
//             if e.generation == self.generation {
//                 if e.depth > depth {
//                     return;
//                 }
//             } else {
//                 self.used_entires += 1;
//             }
//         } else {
//             self.used_entires += 1;
//         }
//         self.entires[index] = Some(TranspositionEntry { hash, generation: self.generation, depth, score, bound });
//     }
// }