use std::iter::IntoIterator;
use std::slice::{Iter, IterMut};

use rand;
use rand::distributions::{IndependentSample, Range};

use rayon::prelude::*;

use super::NQueensSuccessorIter;

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


    /// Returns an Option containing the position of the queen in the given column
    pub fn get_option(&self, column: usize) -> Option<usize> {
        assert!(column < self.size());
        self.queens[column]
    }

    /// Returns a reference to an Option containing the position of the queen in the given column
    pub fn get_option_ref(&self, column: usize) -> &Option<usize> {
        assert!(column < self.size());
        &self.queens[column]
    }


    /// Sets the queen in the given column to the value of `row` directly.
    pub fn set(&mut self, column: usize, row: usize) {
        assert!(column < self.size());
        assert!(row < self.size());

        self.queens[column] = Some(row);
    }

    /// Sets the queen in the given column to the value of `row` as an option.
    pub fn set_option(&mut self, column: usize, row: Option<usize>) {
        assert!(column < self.size());
        if row.is_some() {
            assert!(row.unwrap() < self.size());
        }

        self.queens[column] = row;
    }

    /// Sets the queen in the given column to a position selected uniformly at random
    pub fn set_random(&mut self, column: usize) {
        let mut rng = rand::thread_rng();

        let size = self.size();
        self.set(column, Range::new(0, size).ind_sample(&mut rng));
    }

    /// Removes the queen from the given column if there is one.
    pub fn unset(&mut self, column: usize) {
        assert!(column < self.size());

        self.queens[column] = None;
    }

    /// Returns an iterator over the columns of the board
    pub fn iter(&self) -> Iter<Option<usize>> {
        self.queens.iter()
    }

    /// Returns a mutable iterator over the columns of the board
    pub fn iter_mut(&mut self) -> IterMut<Option<usize>> {
        self.queens.iter_mut()
    }

    /// Creates a struct that implements `Iterator` which provides all possible successors of the
    /// current board configuration. A successor is created by moving a queen in a single column.
    /// It does not place queens in columns that are empty.
    /// 
    /// E.g. if all columns are filled on an 8 by 8 board, the iterator will return 56 boards,
    /// while if no columns are filled it will return 0 boards, and if one is filled it will return
    /// 7.
    pub fn successors_iter(&self) -> NQueensSuccessorIter {
        NQueensSuccessorIter::new(&self)
    }

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
        });

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

    // This logic is basically entirely duplicated, but I don't think there's a good way to
    // deduplicate them because of the short-circuiting we can do in the validity checks vs.
    // counting conflicts

    /// Counts the number of pairs of queens in conflict with each other, including queens passing
    /// through other queens.
    pub fn count_conflicts(&self) -> u32 {
        // NOTE: the `|| 0u32` after the fold calls below is there because the ParallelIterator
        // trait's fold takes an identity function rather than a value

        // do not need to check columns because by definition of our struct we only have one queen
        // per column

        // check rows
        let row_conflicts = self.queens.par_iter().enumerate().fold(|| 0u32, |sum, (i,q)| {
            if q.is_none() { return sum; }
            let q = q.unwrap();
            let qconflicts = self.queens[i+1..].par_iter().fold(|| 0u32, |suminner, q2| {
                if q2.is_none() { return suminner; }

                let q2 = q2.unwrap();
                if q != q2 { return suminner; }
                else { return suminner+1; }
            }).sum();
            return qconflicts+sum;
        }).sum();

        // check diagonals
        // This "row+column are all distinct" test works because if you imagine an addition table,
        // all the elements on a rising diagonal are the same. Similarly, all the elements on a
        // falling diagonal are the same in a subtraction table. Additionally, they are symmetric
        // (up to sign, see below) so we only have to do the computations for (i,j) pairs and not
        // (j,i) as well.
        // Additionally, we don't undercount diagonals when checking both rising and falling
        // diagonals in the same if statement because only one of the checks can fail at a time. If
        // both failed that would mean we have a queen on a rising and falling diagonal equidistant
        // from the queen we're checking, which would mean they are on the same column. We can
        // only have one queen per column so this cannot happen.
        let diagonal_conflicts = self.queens.par_iter().enumerate().fold(|| 0u32, |sum, (i, q)| {
            if q.is_none() { return sum; }
            let q = q.unwrap();
            // optimization so we don't compare (i,j) and (j,i) as noted above
            let qconflicts = self.queens[i+1..].par_iter().enumerate().fold(|| 0u32, |suminner, (j, q2)| {
                if q2.is_none() { return suminner; }
                let q2 = q2.unwrap();
                // The i+1 term appears to account for the shifting that we did in the second
                // enumeration.

                // This should be correct for all useful inputs. We don't actually need the signed
                // answer; we just don't want them to be equal. If the numbers are too large we may
                // be incorrect (i.e. if things are around USIZE_MAX, USIZE_MAX/2 etc.) but that
                // would be way too many queens.
                if (i+q != i+1+j+q2) && (i.wrapping_sub(q) != (i+1+j).wrapping_sub(q2)) {
                    return suminner;
                }
                else {
                    return suminner+1;
                }
            }).sum();
            return qconflicts+sum;
        }).sum();

        return row_conflicts+diagonal_conflicts;
    }
}

use std::ops::Index;
impl Index<usize> for NQueens {
    type Output = usize;

    fn index(&self, column: usize) -> &usize {
        self.get_ref(column)
    }
}

/// Converts a slice of usize to an NQueens with the same number of elements as the slice, such
/// that each column in order has a queen set in the given row. The trait documentation says it
/// must not fail but since TryFrom isn't stabilized we still panic if one of the elements of the
/// slice is larger than its length (i.e. it specifies a queen outside of the boundaries of the
/// board).
impl<T> From<T> for NQueens where T: AsRef<[usize]> {
    fn from(slice: T) -> NQueens {
        let slice = slice.as_ref();
        let mut q = NQueens::new_empty(slice.len());
        for (i,&e) in slice.iter().enumerate() {
            assert!(e < slice.len());
            q.set(i, e);
        }

        q
    }
}

// no `impl IntoIter for NQueens` (i.e. without ref) because it seems useless

impl<'board> IntoIterator for &'board NQueens {
    type Item = &'board Option<usize>;
    type IntoIter = Iter<'board, Option<usize>>;

    fn into_iter(self) -> Iter<'board, Option<usize>> {
        self.queens.iter()
    }
}

impl<'board> IntoIterator for &'board mut NQueens {
    type Item = &'board mut Option<usize>;
    type IntoIter = IterMut<'board, Option<usize>>;

    fn into_iter(self) -> IterMut<'board, Option<usize>> {
        self.queens.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::NQueens;

    // Note that the set(x, y) function is (column, row), so the Q in a diagram on the same line
    // as a set call is not necessarily being set by that call.

    #[test]
    pub fn test_valid_1() {
        let q = NQueens::from([1,3,0,2]);
        // X X Q X
        // Q X X X
        // X X X Q
        // X Q X X

        assert!(q.is_valid() == true);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    #[test]
    pub fn test_valid_2() {
        let q = NQueens::from([2,0,3,1]);
        // X Q X X
        // X X X Q
        // Q X X X
        // X X Q X

        assert!(q.is_valid() == true);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    #[test]
    pub fn test_valid_3() {
        let q = NQueens::from([2,0,3,1,4]);
        // X Q X X X
        // X X X Q X
        // Q X X X X
        // X X Q X X
        // X X X X Q

        assert!(q.is_valid() == true);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    #[test]
    pub fn test_valid_4() {
        let q = NQueens::from([1,4,2,0,3]);
        // X X X Q X
        // Q X X X X
        // X X Q X X
        // X X X X Q
        // X Q X X X

        assert!(q.is_valid() == true);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    // of course we have to test an actual 8 queens
    #[test]
    pub fn test_valid_5() {
        let q = NQueens::from([3,5,7,1,6,0,2,4]);
       // X X X X X Q X X
       // X X X Q X X X X
       // X X X X X X Q X
       // Q X X X X X X X
       // X X X X X X X Q
       // X Q X X X X X X
       // X X X X Q X X X
       // X X Q X X X X X

        assert!(q.is_valid() == true);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    // and another nonsymmetric one
    #[test]
    pub fn test_valid_6() {
        let q = NQueens::from([7,1,4,2,0,6,3,5]);
        // X X X X Q X X X
        // X Q X X X X X X
        // X X X Q X X X X
        // X X X X X X Q X
        // X X Q X X X X X
        // X X X X X X X Q
        // X X X X X Q X X
        // Q X X X X X X X

        assert!(q.is_valid() == true);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }


    #[test]
    pub fn test_horizontals_1() {
        let q = NQueens::from([0,0]);
        // Q Q
        // X X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);
    }

    #[test]
    pub fn test_horizontals_2() {
        let q = NQueens::from([1,1]);
        // X X
        // Q Q
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);
    }

    #[test]
    pub fn test_horizontals_3() {
        let q = NQueens::from([0,2,0]);
        // Q X Q
        // X X X
        // X Q X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);
    }

    #[test]
    pub fn test_horizontals_4() {
        let q = NQueens::from([1,3,0,5,1,4]);
        // X X Q X X X
        // Q X X X Q X
        // X X X X X X
        // X Q X X X X
        // X X X X X Q
        // X X X Q X X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 2, "{} != 2", count);
    }

    #[test]
    pub fn test_horizontals_5() {
        let mut q = NQueens::new_empty(6);
        q.set(0, 0); // Q Q Q Q Q Q
        q.set(1, 0); // X X X X X X
        q.set(2, 0); // X X X X X X
        q.set(3, 0); // X X X X X X
        q.set(4, 0); // X X X X X X
        q.set(5, 0); // X X X X X X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 5+4+3+2+1, "{} != 15", count);
    }

    #[test]
    pub fn test_diagonals_1() {
        let q = NQueens::from([0,1]);
        // Q X
        // X Q
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);
    }

    #[test]
    pub fn test_diagonals_2() {
        let q = NQueens::from([1,0]);
        // X Q
        // Q X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);
    }

    #[test]
    pub fn test_diagonals_3() {
        let q = NQueens::from([0,2,1]);
        // Q X X
        // X X Q
        // X Q X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);
    }

    #[test]
    pub fn test_diagonals_4() {
        let q = NQueens::from([2,0,4,1,3]);
        // X Q X X X
        // X X X Q X
        // Q X X X X
        // X X X X Q
        // X X Q X X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 2, "{} != 2", count);
    }

    #[test]
    pub fn test_diagonals_5() {
        let q = NQueens::from([3,1,4,0,2]);
        // X X X Q X
        // X Q X X X
        // X X X X Q
        // Q X X X X
        // X X Q X X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 2, "{} != 2", count);
    }

    #[test]
    pub fn test_diagonals_6() {
        let q = NQueens::from([0,1,2,3,4]);
        // Q X X X X
        // X Q X X X
        // X X Q X X
        // X X X Q X
        // X X X X Q
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 4+3+2+1, "{} != 10", count);
    }

    #[test]
    pub fn test_diagonals_7() {
        let q = NQueens::from([0,1,2,1,0]);
        // Q X X X Q
        // X Q X Q X
        // X X Q X X
        // X X X X X
        // X X X X X
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == (1+2)+(1+1)+(0+2)+(0+1), "{} != 8", count);
    }

    #[test]
    pub fn test_diagonals_8() {
        let q = NQueens::from([0,1,2,3,4,1,0,7]);
        // Q X X X X X Q X
        // X Q X X X Q X X
        // X X Q X X X X X
        // X X X Q X X X X
        // X X X X Q X X X
        // X X X X X X X X
        // X X X X X X X X
        // X X X X X X X Q

        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == (1+5)+(1+4)+(0+3)+(0+4)+(0+1)+(0+1)+(0+0)+(0+0), "{} != 20", count);
    }

    #[test]
    pub fn test_conflicts_none_between() {
        // Q X Q
        // X X X
        // X X X
        let mut q = NQueens::from([0, 0, 0]);
        q.unset(1);

        assert!(q.is_valid() == false);

        let count = q.count_conflicts();
        assert!(count == 1, "{} != 1", count);

    }
    #[test]
    pub fn test_none_1() {
        let q = NQueens::new_empty(3);
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    #[test]
    pub fn test_none_2() {
        let mut q = NQueens::from([0,2,0]);
        q.unset(2);
        // Q X X
        // X X X
        // X Q X
        
        assert!(q.is_valid() == false);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    #[test]
    pub fn test_iter_empty() {
        let q = NQueens::new_empty(0);
        assert!(q.iter().next() == None);
        let count = q.count_conflicts();
        assert!(count == 0, "{} != 0", count);
    }

    #[test]
    pub fn test_iter_1() {
        let q = NQueens::from([2,0,3,1]);

        let mut qiter = q.iter();
        assert!(qiter.next() == Some(q.get_option_ref(0)));
        assert!(qiter.next() == Some(q.get_option_ref(1)));
        assert!(qiter.next() == Some(q.get_option_ref(2)));
        assert!(qiter.next() == Some(q.get_option_ref(3)));
        assert!(qiter.next() == None);

        let mut qiter = q.iter();
        assert!(qiter.next().unwrap().unwrap() == 2);
        assert!(qiter.next().unwrap().unwrap() == 0);
        assert!(qiter.next().unwrap().unwrap() == 3);
        assert!(qiter.next().unwrap().unwrap() == 1);
        assert!(qiter.next() == None);
    }

    #[test]
    pub fn test_iter_mut_1() {
        let mut b = NQueens::new_empty(4);

        for (i, q) in b.iter_mut().enumerate() {
            *q = Some(4-i);
        }

        assert!(b.get(0) == 4);
        assert!(b.get(1) == 3);
        assert!(b.get(2) == 2);
        assert!(b.get(3) == 1);
    }

    #[test]
    pub fn test_intoiter() {
        let b = NQueens::new_random(4);

        let mut i = 0;
        for q in &b {
            assert!(q.unwrap() == b.get(i));
            i += 1;
        }
    }

    #[test]
    pub fn test_intoiter_mut() {
        let mut b = NQueens::new_empty(4);

        let mut i = 0;
        for q in &mut b {
            *q = Some(i);
            i += 1
        }

        i = 0;
        for q in &b {
            assert!(q.unwrap() == b.get(i));
            i += 1;
        }
    }

    #[test]
    #[should_panic(expected = "assertion failed: e < slice.len()")]
    pub fn test_from_impl_fail() {
        let v = vec![1,2,3,4,5];
        let _ = NQueens::from(v);
    }
} 
