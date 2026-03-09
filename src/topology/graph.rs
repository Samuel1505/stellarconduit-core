use std::collections::{HashMap, HashSet};

use crate::message::types::TopologyUpdate;

pub struct MeshGraph {
    edges: HashMap<[u8; 32], Vec<[u8; 32]>>,
}

impl MeshGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    pub fn apply_update(&mut self, update: &TopologyUpdate) {
        let origin = update.origin_pubkey;
        let mut set: HashSet<[u8; 32]> = HashSet::new();
        for peer in update.directly_connected_peers.iter() {
            if *peer != origin {
                set.insert(*peer);
            }
        }
        let mut list: Vec<[u8; 32]> = set.into_iter().collect();
        list.sort_unstable();
        self.edges.insert(origin, list);
    }

    pub fn get_neighbors(&self, target: &[u8; 32]) -> Option<&Vec<[u8; 32]>> {
        self.edges.get(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(b: u8) -> [u8; 32] {
        [b; 32]
    }

    #[test]
    fn apply_update_filters_self_edges() {
        let origin = pk(1);
        let mut g = MeshGraph::new();
        let update = TopologyUpdate {
            origin_pubkey: origin,
            directly_connected_peers: vec![pk(2), origin, pk(2)],
            hops_to_relay: 5,
        };
        g.apply_update(&update);
        let neighbors = g.get_neighbors(&origin).cloned().unwrap();
        assert!(neighbors.iter().all(|p| *p != origin));
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], pk(2));
    }
}
