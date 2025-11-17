-- Example 7: Zero allocation demonstration
-- All functions here have alloc none, ensuring no dynamic allocation

square: i32 -> i32 !{pure, cpu, alloc none}
square x = x * x

sum_of_squares: i32 -> i32 -> i32 !{pure, cpu, alloc none}
sum_of_squares a b = let sq_a = square a in
                     let sq_b = square b in
                     sq_a + sq_b

main: i32
main = sum_of_squares 3 4
