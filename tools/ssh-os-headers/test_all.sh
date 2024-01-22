#!/usr/bin/env bash

echo "$(date): starting SSH probe generation" > log.txt
echo > results.txt

while read -r line; do
  name="$(echo "$line" | cut -d':' -f1)"
  box="$(echo "$line" | cut -d':' -f2)"
  echo "$(date): test_os.sh $name $box" >> log.txt
  ./test_os.sh "$name" "$box" 2>> log.txt | tee -a results.txt
done < "os-probes.txt"
