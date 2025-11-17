-- Example 4: Arithmetic operations

add: i32 -> i32 -> i32
add x y = x + y

sub: i32 -> i32 -> i32
sub x y = x - y

mul: i32 -> i32 -> i32
mul x y = x * y

div: i32 -> i32 -> i32
div x y = x / y

main: i32
main = let a = add 10 5 in
       let b = sub a 3 in
       let c = mul b 2 in
       div c 4
