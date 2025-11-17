-- Sum benchmark (iterative)
-- Haskell version

sumRange :: Int -> Int -> Int
sumRange start end
  | start > end = 0
  | otherwise   = start + sumRange (start + 1) end

main :: IO ()
main = print $ sumRange 1 10000000
