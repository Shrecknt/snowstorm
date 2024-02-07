use std::hash::Hash;

use crate::RandomWeighted;
use dashmap::DashMap;
use rand::{
    distributions::{DistIter, Distribution, WeightedIndex},
    rngs::ThreadRng,
};
use rayon::prelude::*;

impl<T: Send + Sync + Eq + PartialEq + Hash> RandomWeighted<T> for DashMap<T, usize> {
    async fn select_one_random_weighted(&self) -> T
    where
        T: Copy,
    {
        tokio::task::yield_now().await;
        let weighted = self.weighted().await;
        *self
            .iter()
            .nth(weighted.sample(&mut rand::thread_rng()))
            .unwrap()
            .key()
    }

    async fn select_many_random_weighted(&self, max: usize) -> Vec<T>
    where
        T: Copy,
    {
        tokio::task::yield_now().await;
        let mut res = Vec::new();
        let mut keys = self.weighted_iter().await;
        let orig_keys = self.par_iter().map(|v| *v.key()).collect::<Vec<_>>();
        for _ in 0..orig_keys.len().min(max) {
            let ip = *orig_keys.get(keys.next().unwrap()).unwrap();
            res.push(ip);
        }
        res
    }

    async fn weighted(&self) -> WeightedIndex<usize> {
        tokio::task::yield_now().await;
        WeightedIndex::new(self.iter().map(|v| *v.value())).unwrap()
    }

    async fn weighted_iter(&self) -> DistIter<WeightedIndex<usize>, ThreadRng, usize> {
        tokio::task::yield_now().await;
        self.weighted().await.sample_iter(rand::thread_rng())
    }
}
