-- Example: Print test
-- Demonstrates print functionality with io effect

main: i32 !{io, cpu, alloc none}
main = let a = print 42 in
       let b = print 100 in
       print (a + b)
