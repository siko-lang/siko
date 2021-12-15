#!/bin/bash

SCRIPTDIR=`dirname $0`

cd $SCRIPTDIR

for index in $(seq 1 4000); do

cat > Module${index}.sk << EOL
module Module${index} where

import Std.Util
import Map
    
data FooBar = Alma Int FooBar | Korte Float (Float, Float) (Option Int)
    
data Simple = { alma :: String }
data Simple2 a = { alma :: Option a } deriving Eq, PartialEq
data Simple3 a = extern 

factorial :: Int -> Int
factorial n = if n < 2 then 1 else n * factorial (n-1)

get_stuff1 m k = case get m k of
    Some v -> assert True
    None -> assert False

get_stuff2 m k = case get m k of
    Some v -> assert False
    None -> assert True

map_stuff = do
m :: Map String String <- empty
(m, _) <- insert m "alma" "korte"
get_stuff1 m "alma"
(m, _) <- remove m "alma"
get_stuff2 m "alma"
EOL
done
