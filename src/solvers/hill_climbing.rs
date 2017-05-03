use nqueens_struct::NQueens;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum GradientDescentErr {
    NoSolutionsExist,
    SolutionNotFound,
}

/// Finds a single solution by random gradient descent by generating a random instance and
/// iteratatively looking at the successors of the instance and choosing the one with the fewest
/// number of pairs of queens attacking each other. This either finds a solution or a local
/// minimum, in which case we return a GradientDescentErr.
pub fn hill_climbing_solution(size: usize) -> Result<NQueens, GradientDescentErr> {
    let mut current_iter = NQueens::new_random_permutation(size);
    if size < 2 {
        return Ok(current_iter);
    }

    if size == 2 {
        return Err(GradientDescentErr::NoSolutionsExist);
    }
    if size == 3 {
        return Err(GradientDescentErr::NoSolutionsExist);
    }

    let mut conflicts = current_iter.count_conflicts();
    while conflicts != 0 {
        // iterator cannot be empty when size is nontrivial, so unwrapping is fine
        let (min_succ, min_conflicts) = current_iter.successors_iter()
            .map(|q| {let c = q.count_conflicts(); (q, c)}) // like this because we get borrowck errors otherwise
            .min_by_key(|&(_,c)| c).unwrap();

        // by using >= we prevent getting stuck in loops on plateaus, but may miss solutions
        // directly next to a plateau.
        if min_conflicts >= conflicts {
            return Err(GradientDescentErr::SolutionNotFound);
        }
        else {
            conflicts = min_conflicts;
            current_iter = min_succ;
        }
    }
    // current_iter has zero conflicts
    return Ok(current_iter);
}

#[cfg(test)]
mod test {
    use super::{hill_climbing_solution, GradientDescentErr};

    #[test]
    pub fn test_empty() {
        let solution = hill_climbing_solution(0);
        assert!(solution.is_ok());
        assert!(solution.unwrap().size() == 0);
    }

    #[test]
    pub fn test_2_3() {
        let solution = hill_climbing_solution(2);
        assert!(solution.is_err());
        assert!(solution.unwrap_err() == GradientDescentErr::NoSolutionsExist, "No solutions for size 2 boards");

        let solution = hill_climbing_solution(3);
        assert!(solution.is_err());
        assert!(solution.unwrap_err() == GradientDescentErr::NoSolutionsExist, "No solutions for size 3 boards");
    }

    #[test]
    pub fn test_3_size_8_solutions() {
        let mut solutions = Vec::new();
        while solutions.len() < 3 {
            let solution = hill_climbing_solution(8);
            if solution.is_ok() {
                solutions.push(solution.unwrap());
            }
        }
        for q in solutions {
            assert!(q.is_valid(), "{:?}", q);
        }
    }
}
