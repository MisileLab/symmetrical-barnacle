-- Example 2: Effect violation - this should fail type checking
-- A function with alloc heap cannot be called from alloc none context

foo: i32 -> i32 !{pure, cpu, alloc heap}
foo x = x + 1

bar: i32 -> i32 !{pure, cpu, alloc none}
bar x = foo x

main: i32
main = bar 42
