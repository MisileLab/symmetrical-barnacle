-- Parallel sum benchmark
-- Using concurrent effect for automatic parallelization

compute_square: i32 -> i32 !{pure, cpu, alloc none, concurrent}
compute_square n = n * n

main: i32 !{io, cpu, alloc none}
main = let a = compute_square 100 in
       let b = compute_square 200 in
       let c = compute_square 300 in
       let d = compute_square 400 in
       print (a + b + c + d)
