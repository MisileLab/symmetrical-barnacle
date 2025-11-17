-- Fibonacci benchmark (recursive)
-- Haskell version

fib :: Int -> Int
fib n | n < 2     = n
      | otherwise = fib (n - 1) + fib (n - 2)

main :: IO ()
main = print $ fib 35
