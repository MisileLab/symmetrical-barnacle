-- Example 5: Conditional expressions

abs: i32 -> i32
abs x = if x < 0 then -x else x

max: i32 -> i32 -> i32
max a b = if a > b then a else b

min: i32 -> i32 -> i32
min a b = if a < b then a else b

main: i32
main = let x = abs -42 in
       let y = max x 50 in
       min y 45
