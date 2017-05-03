[![Build Status](https://travis-ci.org/boustrophedon/nqueens-various-rs.svg?branch=master)](https://travis-ci.org/boustrophedon/nqueens-various-rs)

A Rust library featuring structs for representing, doing stuff with, and solving [N-Queens problems](https://en.wikipedia.org/wiki/Eight_queens_puzzle).

I wanted to do this to test implementing some basic AI algorithms and also try out [rayon](https://github.com/nikomatsakis/rayon). 

# TODO
Unify solvers into a trait with something like `solve_one` and `all_solutions` and a unified error type.
