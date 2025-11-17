-- Fibonacci benchmark (recursive)
-- Measures CPU-intensive computation

fib: i32 -> i32 !{pure, cpu, alloc none}
fib n = if n < 2
        then n
        else fib (n - 1) + fib (n - 2)

main: i32 !{io, cpu, alloc none}
main = let result = fib 35 in
       print result
