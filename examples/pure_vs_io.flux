-- Example 6: Pure vs IO effects
-- Pure functions cannot call IO functions

print_number: i32 -> i32 !{io, cpu, alloc none}
print_number x = x

pure_add: i32 -> i32 -> i32 !{pure, cpu, alloc none}
pure_add x y = x + y

main: i32
main = let a = pure_add 10 20 in
       print_number a
