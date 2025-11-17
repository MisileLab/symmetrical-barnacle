// Prime counting benchmark
// Rust version

fn is_prime_helper(n: i32, divisor: i32) -> bool {
    if divisor * divisor > n {
        true
    } else if n % divisor == 0 {
        false
    } else {
        is_prime_helper(n, divisor + 1)
    }
}

fn is_prime(n: i32) -> bool {
    if n < 2 {
        false
    } else {
        is_prime_helper(n, 2)
    }
}

fn count_primes(current: i32, max: i32) -> i32 {
    if current > max {
        0
    } else if is_prime(current) {
        1 + count_primes(current + 1, max)
    } else {
        count_primes(current + 1, max)
    }
}

fn main() {
    let result = count_primes(2, 10000);
    println!("{}", result);
}
