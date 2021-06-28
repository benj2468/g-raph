use std::{collections::binary_heap::Iter, iter::FromIterator};

pub struct GraphStream<S>(S);

impl<S, T> GraphStream<S>
where
    S: Iterator<Item = T>,
{
    pub fn new(iter: S) -> Self {
        Self(iter)
    }
}

impl<S, T> AsRef<S> for GraphStream<S>
where
    S: Iterator<Item = T>,
{
    fn as_ref(&self) -> &S {
        &self.0
    }
}

mod counting;
mod distinct;
mod sampling;
mod sparse_recovery;
