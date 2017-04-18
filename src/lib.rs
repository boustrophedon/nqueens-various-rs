extern crate rand;
extern crate rayon;
extern crate permutohedron;

mod nqueens_struct;
mod nqueens_successor;
pub mod solvers;

pub use nqueens_struct::*;
pub use nqueens_successor::*;
