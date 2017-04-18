use permutohedron;

use rayon::prelude::*;

use nqueens_struct::NQueens;


/// Finds all solutions to the n-queens problem via brute force, by generating all permutations of
/// 1..size and checking each one for validity, and collecting into a Vec.
pub fn brute_force_solutions(size: usize) -> Vec<NQueens> {
    let mut range: Vec<usize> = (0..size).collect();
    let permutations: Vec<NQueens> = permutohedron::Heap::new(&mut range).map(|v| NQueens::from(v)).collect();

    let solutions: Vec<NQueens> = permutations.into_par_iter()
        .filter(|q| q.is_valid())
        .collect();

    solutions
}

#[cfg(test)]
mod test {
    use super::brute_force_solutions;

    #[test]
    pub fn test_brute_force_count_4() {
       assert!(brute_force_solutions(4).iter().count() == 2);
    }

    #[test]
    pub fn test_brute_force_count_5() {
       assert!(brute_force_solutions(5).iter().count() == 10);
    }
}
