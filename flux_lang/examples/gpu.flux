-- Example 3: GPU effect usage
-- GPU functions are marked with gpu execution effect

kernel_add_one: i32 -> i32 !{pure, gpu, alloc none}
kernel_add_one x = x + 1

main: i32
main = kernel_add_one 41
