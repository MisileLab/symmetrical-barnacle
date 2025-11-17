-- Parallel Fibonacci benchmark
-- Demonstrates automatic parallelization of pure functions

fib: i32 -> i32 !{pure, cpu, alloc none}
fib n = if n < 2
        then n
        else fib (n - 1) + fib (n - 2)

-- Using parallel execution for independent computations
main: i32 !{io, cpu, alloc none}
main = let a = fib 30 in
       let b = fib 30 in
       let c = fib 30 in
       let d = fib 30 in
       print (a + b + c + d)
