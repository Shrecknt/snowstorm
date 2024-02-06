use super::Top;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

impl<T> Top for HashMap<T, usize>
where
    T: Hash + Ord + Copy,
{
    fn top(&self, n: usize) -> Self {
        let btree_map = self
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect::<BTreeMap<usize, T>>();
        let mut res = HashMap::new();
        let mut map_iter = btree_map.iter().rev();
        for _ in 0..btree_map.len().min(n) {
            let (v, k) = map_iter.next().unwrap();
            res.insert(*k, *v);
        }
        res
    }
}
