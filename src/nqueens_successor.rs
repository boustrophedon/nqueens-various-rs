use super::NQueens;

/// An iterator which returns a sequence of successors of the given board. A successor is defined
/// as a board which can be obtained by changing the location of a single queen within a column.
/// Columns which are `None`, that is, without a queen, are not changed.
pub struct NQueensSuccessorIter<'original> {
    original: &'original NQueens,
    current_config: NQueens,
    current_column: usize,
    done: bool,
}

impl<'original> NQueensSuccessorIter<'original> {
    pub fn new(original: &NQueens) -> NQueensSuccessorIter {
        let mut current_config = original.clone();
        let current_column = 0;
        if original.size() != 0 {
            current_config.unset(current_column);
        }
        NQueensSuccessorIter {
            original: original,
            current_config: current_config,
            current_column: current_column,
            done: false,
        }
    }
}

// so here's what we need to do:
// - if the original column is None, skip it
// - we need to be able to tell if we have started on the column: this can be done by comparing it
// to the original and making sure that we never leave the current column in the same state as the
// original after starting
// - we need to be able to say that we are done with the current column, probably via the same
// mechanism as skipping a None
// - when we are done, we need to reset the current column to the original
// - when we reach the end we need to check, probably in the outer loop, that if we just finished,
// whether we're on the last column

impl<'original> Iterator for NQueensSuccessorIter<'original> {
    type Item = NQueens;

    fn next(&mut self) -> Option<NQueens> {
        // no successors of a 1x1 board either way - either there's already a queen there or there
        // isn't, and we don't add queens to make a successor. 0 is trivial.
        if self.original.size() <= 1 {
            return None
        }

        while self.current_column < self.original.size() {
            let current_col = self.current_config.get_option(self.current_column);
            let orig_col = self.original.get_option(self.current_column);
            let next_col = next_column(current_col, orig_col, self.original.size());

            if next_col.is_some() {
                self.current_config.set(self.current_column, next_col.unwrap());
                return Some(self.current_config.clone());
            }

            // either we've reached the end of the current column in the current configuration or
            // it was None to begin with in the original
            else {
                // reset the current column to the original and increment
                self.current_config.set_option(self.current_column, self.original.get_option(self.current_column));
                self.current_column = self.current_column + 1;
                // unset the new current column to indicate we're starting a new column
                if self.current_column < self.original.size() {
                    self.current_config.unset(self.current_column);
                }
            }
        }
        // if we get through the loop without returning, we have hit all the successors and are
        // done
        self.done = true;
        return None;
    }
}

fn next_column(col: Option<usize>, orig: Option<usize>, size: usize) -> Option<usize> {
    debug_assert!(size>0); // this is checked in NQueensSuccessorIter::next
    match (col, orig) {
        // case we are currently in a row
        (Some(row), Some(orow)) => {
            let mut next = row+1;
            if next == orow {
                next = next+1;
            }

            let output;
            if next >= size {
                output = None;
            }
            else {
                output = Some(next);
            }
            output
        },
        // case we are starting a row
        (None, Some(orow)) => {
            if orow == 0 {
                Some(1)
            }
            else {
                Some(0)
            }
        },
        // other cases both have orig as None, so we skip
        _ => None
    }
}
#[cfg(test)]
mod test {
    use NQueens;
    #[test]
    pub fn test_successors_size0() {
        let q = NQueens::new_empty(0);
        let mut qsucc = q.successors_iter();

        assert!(qsucc.next().is_none());
        assert!(qsucc.next().is_none());
    }

    #[test]
    pub fn test_successors_size1() {
        let mut q = NQueens::new_empty(1);
        {
        let mut qsucc = q.successors_iter();

        assert!(qsucc.next().is_none());
        assert!(qsucc.next().is_none());
        }
        {
        q.set(0, 0);
        let mut qsucc = q.successors_iter();

        assert!(qsucc.next().is_none());
        }
    }

    #[test]
    pub fn test_successors_size2_1() {
        let mut q = NQueens::new_empty(2);
        q.set(0, 0); // Q X
        q.set(1, 1); // X Q
        let mut qsucc = q.successors_iter();

        let c = qsucc.next().unwrap();
        assert!(c.get(0) == 1); // X X
        assert!(c.get(1) == 1); // Q Q

        let c = qsucc.next().unwrap();
        assert!(c.get(0) == 0); // Q Q
        assert!(c.get(1) == 0); // X X

        assert!(qsucc.next().is_none());
        assert!(qsucc.next().is_none());
    }

    #[test]
    pub fn test_successors_size2_empty_cols() {
        let mut q = NQueens::new_empty(2);
        {
        q.set(1, 0);
        let mut qsucc = q.successors_iter();

        let c = qsucc.next().unwrap();
        assert!(c.get_option(0).is_none());
        assert!(c.get(1) == 1);

        assert!(qsucc.next().is_none());
        assert!(qsucc.next().is_none());
        }
        {
        q.unset(1);
        q.set(0, 1);
        let mut qsucc = q.successors_iter();

        let c = qsucc.next().unwrap();
        assert!(c.get(0) == 0);
        assert!(c.get_option(1).is_none());

        assert!(qsucc.next().is_none());
        assert!(qsucc.next().is_none());
        }
    }

    #[test]
    pub fn test_successors_size4_empty() {
        let q = NQueens::new_empty(4);
        let mut qsucc = q.successors_iter();
        assert!(qsucc.next().is_none());
        assert!(qsucc.next().is_none());
    }

    #[test]
    pub fn test_successors_size8_full_1() {
        let mut q = NQueens::new_empty(8);
        q.set(0, 7); // X X X X Q X X X
        q.set(1, 1); // X Q X X X X X X
        q.set(2, 4); // X X X Q X X X X
        q.set(3, 2); // X X X X X X Q X
        q.set(4, 0); // X X Q X X X X X
        q.set(5, 6); // X X X X X X X Q
        q.set(6, 3); // X X X X X Q X X
        q.set(7, 5); // Q X X X X X X X

        let count = q.successors_iter().count();
        assert!(count == 7*8); // seven empty spaces in each column, 8 columns
    }

    #[test]
    pub fn test_successors_size8_1empty_first() {
        let mut q = NQueens::new_empty(8);
        // q.set(0, 7); X X X X Q X X X
        q.set(1, 1); // X Q X X X X X X
        q.set(2, 4); // X X X Q X X X X
        q.set(3, 2); // X X X X X X Q X
        q.set(4, 0); // X X Q X X X X X
        q.set(5, 6); // X X X X X X X Q
        q.set(6, 3); // X X X X X Q X X
        q.set(7, 5); // X X X X X X X X

        let count = q.successors_iter().count();
        assert!(count == 7*7); // seven empty spaces in each column, 7 nonempty columns
    }

    #[test]
    pub fn test_successors_size8_1empty_last() {
        let mut q = NQueens::new_empty(8);
        q.set(0, 7); // X X X X Q X X X
        q.set(1, 1); // X Q X X X X X X
        q.set(2, 4); // X X X Q X X X X
        q.set(3, 2); // X X X X X X Q X
        q.set(4, 0); // X X Q X X X X X
        q.set(5, 6); // X X X X X X X X
        q.set(6, 3); // X X X X X Q X X
        // q.set(7, 5); Q X X X X X X X

        let count = q.successors_iter().count();
        assert!(count == 7*7); // seven empty spaces in each column, 7 nonempty columns
    }

    #[test]
    pub fn test_successors_size8_1empty_middle() {
        let mut q = NQueens::new_empty(8);
        q.set(0, 7); // X X X X Q X X X
        q.set(1, 1); // X Q X X X X X X
        // q.set(2, 4); X X X Q X X X X
        q.set(3, 2); // X X X X X X Q X
        q.set(4, 0); // X X Q X X X X X
        q.set(5, 6); // X X X X X X X Q
        q.set(6, 3); // X X X X X Q X X
        q.set(7, 5); // Q X X X X X X X

        let count = q.successors_iter().count();
        assert!(count == 7*7); // seven empty spaces in each column, 7 nonempty columns
    }

    #[test]
    pub fn test_successors_size8_2empty_middle() {
        let mut q = NQueens::new_empty(8);
        q.set(0, 7); // X X X X Q X X X
        q.set(1, 1); // X Q X X X X X X
        // q.set(2, 4); X X X Q X X X X
        // q.set(3, 2); X X X X X X Q X
        q.set(4, 0); // X X Q X X X X X
        q.set(5, 6); // X X X X X X X Q
        q.set(6, 3); // X X X X X Q X X
        q.set(7, 5); // Q X X X X X X X

        let count = q.successors_iter().count();
        assert!(count == 7*6); // seven empty spaces in each column, 6 nonempty columns
    }

    #[test]
    pub fn test_successors_size8_7empty_middle() {
        let mut q = NQueens::new_empty(8);
        q.set(4, 0);

        let count = q.successors_iter().count();
        assert!(count == 7);
    }
}
