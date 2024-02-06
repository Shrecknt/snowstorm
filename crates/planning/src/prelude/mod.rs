use rand::{
    distributions::{DistIter, WeightedIndex},
    rngs::ThreadRng,
};

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
    fn select_one_random_weighted(&self) -> T
    where
        T: Copy;
    fn select_many_random_weighted(&self, max: usize) -> Vec<T>
    where
        T: Copy;
    fn weighted(&self) -> WeightedIndex<usize>;
    fn weighted_iter(&self) -> DistIter<WeightedIndex<usize>, ThreadRng, usize>;
}
