-- Parallel Fibonacci with threshold-based parallelization
-- Only parallelizes for n >= 30 to avoid overhead
-- Should beat single-threaded Rust for large inputs!

-- Sequential Fibonacci for small inputs
fib_seq: i32 -> i32 !{pure, cpu, alloc none, nonConcurrent}
fib_seq n = if n < 2
            then n
            else fib_seq (n - 1) + fib_seq (n - 2)

-- Parallel Fibonacci with threshold
-- For n >= 30: compute left and right branches in parallel
-- For n < 30: use sequential version (less overhead)
fib: i32 -> i32 !{pure, cpu, alloc none}
fib n = if n < 30
        then fib_seq n
        else if n < 2
             then n
             else let left = fib (n - 1) in
                  let right = fib (n - 2) in
                  left + right

main: i32 !{io, cpu, alloc none}
main = let result = fib 40 in
       print result
