use rand;
use rand::distributions::{IndependentSample, Range};

use rayon::prelude::*;


#[derive(Debug, Clone)]
pub struct NQueens {
    queens: Vec<Option<usize>>,
}

impl NQueens {
    /// Creates a new board of size `usize` with None in each position
    pub fn new_empty(size: usize) -> NQueens {
        let mut queens = Vec::new();
        for _ in 0..size {
            queens.push(None);
        }

        NQueens {
            queens: queens,
        }
    }

    /// Creates a new board of size `usize` with each queen's position selected uniformly at random
    pub fn new_random(size: usize) -> NQueens {
        let mut queens = Vec::new();

        let mut rng = rand::thread_rng();
        for _ in 0..size {
            queens.push(Some(Range::new(0, size).ind_sample(&mut rng)));
        }

        NQueens {
            queens: queens,
        }
    }

    /// Returns the size of the board i.e. the width and height, which are equal
    pub fn size(&self) -> usize {
        self.queens.len()
    }

    /// Returns true if queen's position is set in the given column
    pub fn is_set(&self, column: usize) -> bool {
        assert!(column < self.size());
        self.queens[column].is_some()
    }

    /// Returns the position of the queen in the given column. Panics if queen is not set in that
    /// column
    pub fn get(&self, column: usize) -> usize {
        assert!(column < self.size());
        self.queens[column].unwrap()
    }

    /// Returns a reference to the position of the queen in the given column. Panics if queen is
    /// not set in that column
    pub fn get_ref(&self, column: usize) -> &usize {
        assert!(column < self.size());
        self.queens[column].as_ref().unwrap()
    }


    /// Returns an Option contains the position of the queen in the given column
    pub fn get_option(&self, column: usize) -> Option<usize> {
        assert!(column < self.size());
        self.queens[column]
    }

    /// Sets the queen in the given column to the value of `row`.
    pub fn set(&mut self, column: usize, row: usize) {
        assert!(column < self.size());
        assert!(row < self.size());

        self.queens[column] = Some(row);
    }

    /// Sets the queen in the given column to a position selected uniformly at random
    pub fn set_random(&mut self, column: usize) {
        let mut rng = rand::thread_rng();

        let size = self.size();
        self.set(column, Range::new(0, size).ind_sample(&mut rng));
    }

    /// Creates a struct that implements `Iterator` which provides all possible successors of the
    /// current board configuration. A successor is created by moving or placing a queen in a
    /// single column.
    /// 
    /// E.g. if all queens are filled on an 8 by 8 board, the iterator will return 56 boards,
    /// while if no queens are filled the iterator will iterate over 64 boards.
    //pub fn to_successors_iter(&self) -> NQueensSuccessorIter {
    //    NQueensSuccessorIter::new(&self)
    //}

    /// Checks if the current configuration of the board is a valid solution
    pub fn is_valid(&self) -> bool {
        // Check if all entries are not None; this lets us use unwrap everywhere and provides a
        // quick exit.
        let full_board = self.queens.par_iter()
                             .all(|q| !q.is_none());
        if !full_board {
            return false;
        }

        // do not need to check columns because by definition of our struct we only have one queen
        // per column

        // check rows
        let all_rows_distinct = self.queens.par_iter().enumerate().all( |(i,q)| {
            let q = q.unwrap();
            self.queens[i+1..].par_iter().all( |q2| {
                let q2 = q2.unwrap();
                q != q2
            })
        } );

        if !all_rows_distinct {
            return false;
        }

        // check diagonals
        // This "row+column are all distinct" test works because if you imagine an addition table,
        // all the elements on a rising diagonal are the same. Similarly, all the elements on a
        // falling diagonal are the same in a subtraction table. Additionally, they are symmetric
        // (up to sign, see below) so we only have to do the computations for (i,j) pairs and not
        // (j,i) as well.
        let all_diagonals_distinct = self.queens.par_iter().enumerate().all( |(i, q)| {
            let q = q.unwrap();
            // optimization so we don't compare (i,j) and (j,i) as noted above
            self.queens[i+1..].par_iter().enumerate().all( |(j, q2)| {
                let q2 = q2.unwrap();
                // The i+1 term appears to account for the shifting that we did in the second
                // enumeration.
                
                // This should be correct for all useful inputs. We don't actually need the signed
                // answer; we just don't want them to be equal. If the numbers are too large we may
                // be incorrect (i.e. if things are around USIZE_MAX, USIZE_MAX/2 etc.) but that
                // would be way too many queens.
                (i+q != i+1+j+q2) && (i.wrapping_sub(q) != (i+1+j).wrapping_sub(q2))
            })
        });

        if !all_diagonals_distinct {
            return false;
        }

        // if above checks pass, this is a valid solution and we return true
        return true;
    }
}

use std::ops::Index;
impl Index<usize> for NQueens {
    type Output = usize;

    fn index(&self, column: usize) -> &usize {
        self.get_ref(column)
    }
}

#[cfg(test)]
mod test {
    use super::NQueens;

    // Note that the set(x, y) function is (column, row), so the Q in a diagram on the same line
    // as a set call is not necessarily being set by that call.

    #[test]
    pub fn test_valid_1() {
        let mut q = NQueens::new_empty(4);
        q.set(0, 1); // X X Q X
        q.set(1, 3); // Q X X X
        q.set(2, 0); // X X X Q
        q.set(3, 2); // X Q X X

        assert!(q.is_valid() == true);
    }

    #[test]
    pub fn test_valid_2() {
        let mut q = NQueens::new_empty(4);
        q.set(0, 2); // X Q X X
        q.set(1, 0); // X X X Q
        q.set(2, 3); // Q X X X
        q.set(3, 1); // X X Q X

        assert!(q.is_valid() == true);
    }

    #[test]
    pub fn test_valid_3() {
        let mut q = NQueens::new_empty(5);
        q.set(0, 2); // X Q X X X
        q.set(1, 0); // X X X Q X
        q.set(2, 3); // Q X X X X
        q.set(3, 1); // X X Q X X
        q.set(4, 4); // X X X X Q

        assert!(q.is_valid() == true);
    }

    #[test]
    pub fn test_valid_4() {
        let mut q = NQueens::new_empty(5);
        q.set(0, 1); // X X X Q X
        q.set(1, 4); // Q X X X X
        q.set(2, 2); // X X Q X X
        q.set(3, 0); // X X X X Q
        q.set(4, 3); // X Q X X X

        assert!(q.is_valid() == true);
    }

    // of course we have to test an actual 8 queens
    #[test]
    pub fn test_valid_5() {
        let mut q = NQueens::new_empty(8);
        q.set(0, 3); // X X X X X Q X X
        q.set(1, 5); // X X X Q X X X X
        q.set(2, 7); // X X X X X X Q X
        q.set(3, 1); // Q X X X X X X X
        q.set(4, 6); // X X X X X X X Q
        q.set(5, 0); // X Q X X X X X X
        q.set(6, 2); // X X X X Q X X X
        q.set(7, 4); // X X Q X X X X X

        assert!(q.is_valid() == true);
    }

    // and another nonsymmetric one
    #[test]
    pub fn test_valid_6() {
        let mut q = NQueens::new_empty(8);
        q.set(0, 7); // X X X X Q X X X
        q.set(1, 1); // X Q X X X X X X
        q.set(2, 4); // X X X Q X X X X
        q.set(3, 2); // X X X X X X Q X
        q.set(4, 0); // X X Q X X X X X
        q.set(5, 6); // X X X X X X X Q
        q.set(6, 3); // X X X X X Q X X
        q.set(7, 5); // Q X X X X X X X

        assert!(q.is_valid() == true);
    }


    #[test]
    pub fn test_horizontals_1() {
        let mut q = NQueens::new_empty(2);
        q.set(0, 0); // Q Q
        q.set(1, 0); // X X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_horizontals_2() {
        let mut q = NQueens::new_empty(2);
        q.set(0, 0); // X X
        q.set(1, 0); // Q Q
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_horizontals_3() {
        let mut q = NQueens::new_empty(3);
        q.set(0, 0); // Q X Q
        q.set(1, 2); // X X X
        q.set(2, 0); // X Q X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_horizontals_4() {
        let mut q = NQueens::new_empty(6);
        q.set(0, 0); // X X Q X X X
        q.set(1, 2); // Q X X X Q X
        q.set(2, 0); // X X X X X X
        q.set(2, 0); // X Q X X X X
        q.set(2, 0); // X X X X X Q
        q.set(2, 0); // X X X Q X X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_diagonals_1() {
        let mut q = NQueens::new_empty(2);
        q.set(0, 0); // Q X
        q.set(1, 1); // X Q
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_diagonals_2() {
        let mut q = NQueens::new_empty(2);
        q.set(0, 1); // X Q
        q.set(1, 0); // Q X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_diagonals_3() {
        let mut q = NQueens::new_empty(3);
        q.set(0, 0); // Q X X
        q.set(1, 2); // X X Q
        q.set(2, 1); // X Q X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_diagonals_4() {
        let mut q = NQueens::new_empty(5);
        q.set(0, 2); // X Q X X X
        q.set(1, 0); // X X X Q X
        q.set(2, 4); // Q X X X X
        q.set(3, 1); // X X X X Q
        q.set(3, 3); // X X Q X X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_diagonals_5() {
        let mut q = NQueens::new_empty(5);
        q.set(0, 3); // X X X Q X
        q.set(1, 1); // X Q X X X
        q.set(2, 4); // X X X X Q
        q.set(3, 0); // Q X X X X
        q.set(3, 2); // X X Q X X
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_none_1() {
        let q = NQueens::new_empty(3);
        assert!(q.is_valid() == false);
    }

    #[test]
    pub fn test_none_2() {
        let mut q = NQueens::new_empty(3);
        q.set(0, 0); // Q X X
        q.set(1, 2); // X X X
                     // X Q X
        
        assert!(q.is_valid() == false);
    }
} 
