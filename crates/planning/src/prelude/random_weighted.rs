use crate::RandomWeighted;
use rand::{
    distributions::{DistIter, Distribution, WeightedIndex},
    rngs::ThreadRng,
};
use std::collections::HashMap;

impl<T> RandomWeighted<T> for HashMap<T, usize> {
    fn select_one_random_weighted(&self) -> T
    where
        T: Copy,
    {
        let weighted = self.weighted();
        *self
            .keys()
            .nth(weighted.sample(&mut rand::thread_rng()))
            .unwrap()
    }

    fn select_many_random_weighted(&self, max: usize) -> Vec<T>
    where
        T: Copy,
    {
        let mut res = Vec::new();
        let mut keys = self.weighted_iter();
        let orig_keys = self.keys().collect::<Vec<_>>();
        for _ in 0..orig_keys.len().min(max) {
            let ip = **orig_keys.get(keys.next().unwrap()).unwrap();
            res.push(ip);
        }
        res
    }

    fn weighted(&self) -> WeightedIndex<usize> {
        WeightedIndex::new(self.values()).unwrap()
    }

    fn weighted_iter(&self) -> DistIter<WeightedIndex<usize>, ThreadRng, usize> {
        self.weighted().sample_iter(rand::thread_rng())
    }
}
