//! Graph Streaming Algorithms
//!
//! Many of the functions here are implemented based off of the lecture notes from Dartmouth's [CS35 Spring 2020 Lecture Notes](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf)

#[derive(Debug)]
pub enum Stream<S, Q> {
    InStream(S),
    Queried(Q),
}

impl<S, Q> Stream<S, Q> {
    /// Panics if the stream as already been consumed.
    fn as_stream(&mut self) -> &mut S {
        match self {
            Self::InStream(s) => s,
            _ => panic!("Cannot capture a stream that has already been consumed"),
        }
    }
}

pub trait Query<T> {
    fn query(self) -> T;
}

impl<S, Q> Stream<S, Q>
where
    S: Query<Q>,
{
    fn query(self) -> Q {
        match self {
            Self::InStream(stream) => stream.query(),
            Self::Queried(res) => res,
        }
    }
}

pub mod coloring;
mod counting;
mod distinct;
pub mod sampling;
pub mod sparse_recovery;
