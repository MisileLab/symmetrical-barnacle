-- Smart parallel Fibonacci with threshold
-- Uses simple threshold: below 25 returns early, above 25 parallelizes

fib: i32 -> i32 !{pure, cpu, alloc none}
fib n = if n < 2
        then n
        else if n < 25
             -- Small problem: compute inline without parallelization
             then if n < 2 then n else fib (n - 1) + fib (n - 2)
             -- Large problem: automatic parallelization kicks in!
             else fib (n - 1) + fib (n - 2)

main: i32 !{io, cpu, alloc none}
main = let result = fib 35 in
       print result
