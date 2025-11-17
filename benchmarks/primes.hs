-- Prime counting benchmark
-- Haskell version

isPrime' :: Int -> Int -> Bool
isPrime' n divisor
  | divisor * divisor > n = True
  | n `mod` divisor == 0  = False
  | otherwise             = isPrime' n (divisor + 1)

isPrime :: Int -> Bool
isPrime n | n < 2     = False
          | otherwise = isPrime' n 2

countPrimes :: Int -> Int -> Int
countPrimes current max
  | current > max = 0
  | isPrime current = 1 + countPrimes (current + 1) max
  | otherwise       = countPrimes (current + 1) max

main :: IO ()
main = print $ countPrimes 2 10000
