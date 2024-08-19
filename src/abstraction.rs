use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::{fs::File, io::BufReader};

use poker_indexer::Indexer;

use crate::tables::load;

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub i: u32,
    pub n: u32,
    pub r: u32,
    pub t: u32,
    pub a: char,

    pub s0: u32,
    pub s1: u32,

    pub children: Vec<usize>,
}

pub struct Abstraction {
    indexer_0: Indexer,
    indexer_1: Indexer,
    indexer_2: Indexer,
    indexer_3: Indexer,

    cluster_1: Vec<u16>,
    cluster_2: Vec<u16>,
    cluster_3: Vec<u16>,

    tree: Vec<Node>,
}

impl Abstraction {
    pub fn new(path: String, file: String) -> Self {
        let tree = serde_json::from_reader(BufReader::new(File::open(file).unwrap())).unwrap();

        Self {
            indexer_0: Indexer::new(vec![2]),
            indexer_1: Indexer::new(vec![2, 3]),
            indexer_2: Indexer::new(vec![2, 4]),
            indexer_3: Indexer::new(vec![2, 5]),

            cluster_1: load(&(path.clone() + "flop.bin")),
            cluster_2: load(&(path.clone() + "turn.bin")),
            cluster_3: load(&(path.clone() + "river.bin")),

            tree,
        }
    }

    pub fn size(&self) -> u32 {
        self.tree[0].i
    }

    pub fn root(&self) -> &Node {
        &self.tree[0]
    }

    pub fn next(&self, node: &Node, action: usize) -> &Node {
        &self.tree[node.children[action]]
    }

    pub fn index(&self, cards: SmallVec<[u64; 4]>, node: &Node) -> u32 {
        node.i
            - match node.r {
                0 => self.indexer_0.index(cards),
                1 => self.cluster_1[self.indexer_1.index(cards) as usize] as u32,
                2 => self.cluster_2[self.indexer_2.index(cards) as usize] as u32,
                3 => self.cluster_3[self.indexer_3.index(cards) as usize] as u32,
                _ => panic!("invalid round"),
            }
            - 1
    }
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use super::*;

    #[test]
    fn test_size() {
        let abstraction = Abstraction::new(
            "data/tables/".to_string(),
            "data/tables/action-tree.json".to_string(),
        );

        let actions = vec![
            1, 0, 2, 3
        ];

        let mut node = abstraction.root();
        for action in actions {
            node = abstraction.next(node, action);
            print!("{}", node.a);
        }
        println!();

        println!("r = {}; s0 = {}; s1 = {}", node.r, node.s0, node.s1);

        let cards = smallvec![1 | 1 << 13, 1 << 26 | 1 << 12 | 1 << 11];

        println!("index: {}", abstraction.index(cards, node));
    }
}
