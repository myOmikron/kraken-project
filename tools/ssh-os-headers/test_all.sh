#!/usr/bin/env bash

echo "$(date): starting SSH probe generation" > log.txt
mkdir -p results

cleanup ()
{
  echo "Aborting OS test..." >&2
  kill -s SIGINT $!
  wait $!
  exit 0
}

trap cleanup SIGINT SIGTERM

while read -r line; do
  [[ "$line" =~ ^#.* ]] && continue
  name="$(echo "$line" | cut -d':' -f1)"
  box="$(echo "$line" | cut -d':' -f2)"
  echo "$(date): test_os.sh $name $box" | tee -a log.txt >&2
  # Make sure command does not hijack stdin
  echo "" | ./test_os.sh "$name" "$box" 2>> log.txt | tee "results/$name.txt" &
  wait $!
  sleep 0.1
done < "os-probes.txt"
