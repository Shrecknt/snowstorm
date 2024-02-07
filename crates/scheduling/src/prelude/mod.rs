use rand::{
    distributions::{DistIter, WeightedIndex},
    rngs::ThreadRng,
};
use std::future::Future;

mod dedupe;
mod random_weighted;
mod top;

pub trait Dedupe {
    fn dedupe(&self) -> Self;
}

pub trait Top {
    fn top(&self, n: usize) -> Self;
}

pub trait RandomWeighted<T> {
    fn select_one_random_weighted(&self) -> impl Future<Output = T> + Send
    where
        T: Copy;
    fn select_many_random_weighted(
        &self,
        max: usize,
    ) -> impl std::future::Future<Output = Vec<T>> + Send
    where
        T: Copy;
    fn weighted(&self) -> impl std::future::Future<Output = WeightedIndex<usize>> + Send;
    fn weighted_iter(
        &self,
    ) -> impl std::future::Future<Output = DistIter<WeightedIndex<usize>, ThreadRng, usize>> + Send;
}
