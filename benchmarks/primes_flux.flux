-- Prime counting benchmark
-- Counts primes up to N

is_prime: i32 -> i32 -> i32 !{pure, cpu, alloc none}
is_prime n divisor = if divisor * divisor > n
                     then 1
                     else if n % divisor == 0
                          then 0
                          else is_prime n (divisor + 1)

check_prime: i32 -> i32 !{pure, cpu, alloc none}
check_prime n = if n < 2
                then 0
                else is_prime n 2

count_primes: i32 -> i32 -> i32 !{pure, cpu, alloc none}
count_primes current max = if current > max
                           then 0
                           else check_prime current + count_primes (current + 1) max

main: i32 !{io, cpu, alloc none}
main = let result = count_primes 2 10000 in
       print result
