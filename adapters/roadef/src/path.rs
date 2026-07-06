use std::collections::HashSet;

pub type NodeId = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum PathState {
    Uninitialized,
    Explicit(Vec<NodeId>),
}

#[derive(Debug, Clone)]
pub struct SrPathBit {
    pub state: PathState,
    pub mask: HashSet<(NodeId, NodeId)>,
}

impl SrPathBit {
    pub fn new_uninitialized() -> Self {
        Self {
            state: PathState::Uninitialized,
            mask: HashSet::new(),
        }
    }

    pub fn new_explicit(source: NodeId, target: NodeId, waypoints: &[NodeId]) -> Self {
        let mut full_path = Vec::with_capacity(waypoints.len() + 2);
        full_path.push(source);
        full_path.extend_from_slice(waypoints);
        full_path.push(target);

        let mut mask = HashSet::new();
        for i in 0..full_path.len() - 1 {
            let u = full_path[i];
            let v = full_path[i + 1];
            mask.insert((u, v));
        }

        Self {
            state: PathState::Explicit(full_path),
            mask,
        }
    }

    pub fn is_uninitialized(&self) -> bool {
        matches!(self.state, PathState::Uninitialized)
    }

    pub fn segment_num(&self) -> usize {
        match &self.state {
            PathState::Uninitialized => 0,
            PathState::Explicit(path) => path.len(),
        }
    }

    pub fn dist(&self, other: &SrPathBit) -> usize {
        if self.is_uninitialized() {
            return other.segment_num();
        }
        if other.is_uninitialized() {
            return self.segment_num();
        }

        // Both are initialized
        // Distance is the size of the symmetric difference of masks
        let diff1 = self.mask.difference(&other.mask).count();
        let diff2 = other.mask.difference(&self.mask).count();
        diff1 + diff2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninitialized_dist() {
        let uninit = SrPathBit::new_uninitialized();
        let explicit = SrPathBit::new_explicit(1, 4, &[2, 3]); // path: [1, 2, 3, 4], segments = 4
        
        assert_eq!(uninit.dist(&explicit), 4);
        assert_eq!(explicit.dist(&uninit), 4);
    }

    #[test]
    fn test_explicit_dist() {
        let p1 = SrPathBit::new_explicit(1, 9, &[5, 7]); // path: [1, 5, 7, 9], transitions: (1,5), (5,7), (7,9)
        let p2 = SrPathBit::new_explicit(1, 9, &[6, 7]); // path: [1, 6, 7, 9], transitions: (1,6), (6,7), (7,9)

        // p1 mask: (1,5), (5,7), (7,9)
        // p2 mask: (1,6), (6,7), (7,9)
        // symmetric difference: (1,5), (5,7) and (1,6), (6,7) -> 4
        assert_eq!(p1.dist(&p2), 4);
    }

    #[test]
    fn test_empty_waypoints_dist() {
        // "Empty" path in JSON still has source and target
        let p_empty = SrPathBit::new_explicit(1, 9, &[]); // transitions: (1,9)
        let p_waypoints = SrPathBit::new_explicit(1, 9, &[5]); // transitions: (1,5), (5,9)

        // p_empty mask: (1,9)
        // p_waypoints mask: (1,5), (5,9)
        // diff: 3
        assert_eq!(p_empty.dist(&p_waypoints), 3);
    }

    #[test]
    fn test_case_b_duplicate_transitions() {
        // Case B: [1, 5, 5, 9] vs [1, 5, 9]
        let p_dup = SrPathBit::new_explicit(1, 9, &[5, 5]); // transitions: (1,5), (5,5), (5,9)
        let p_no_dup = SrPathBit::new_explicit(1, 9, &[5]); // transitions: (1,5), (5,9)
        
        // Distance should be 1 (the (5,5) transition)
        assert_eq!(p_dup.dist(&p_no_dup), 1);
    }

    #[test]
    fn test_case_c_uninitialized_vs_empty() {
        // Case C: Missing demand (uninitialized) vs Empty JSON array `[]`
        let p_missing = SrPathBit::new_uninitialized();
        let p_empty_json = SrPathBit::new_explicit(1, 9, &[]); // path: [1, 9], segments: 2
        
        // If one is uninitialized, distance is other.segment_num()
        // segment_num() for p_empty_json is 2 (source + target)
        assert_eq!(p_missing.dist(&p_empty_json), 2);
    }
}
