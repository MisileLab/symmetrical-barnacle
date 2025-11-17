-- Sum benchmark (iterative)
-- Measures simple loop performance

sum_range: i32 -> i32 -> i32 !{pure, cpu, alloc none}
sum_range start end = if start > end
                      then 0
                      else start + sum_range (start + 1) end

main: i32 !{io, cpu, alloc none}
main = let result = sum_range 1 10000000 in
       print result
