use super::Top;
use dashmap::DashMap;
use std::{collections::BTreeMap, hash::Hash};

impl<T> Top for DashMap<T, usize>
where
    T: Hash + Ord + Copy,
{
    fn top(&self, n: usize) -> Self {
        let btree_map = self
            .iter()
            .map(|pair| (*pair.value(), *pair.key()))
            .collect::<BTreeMap<usize, T>>();
        let res = DashMap::new();
        let mut map_iter = btree_map.iter().rev();
        for _ in 0..btree_map.len().min(n) {
            let (v, k) = map_iter.next().unwrap();
            res.insert(*k, *v);
        }
        res
    }
}
