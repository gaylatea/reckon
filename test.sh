#!/bin/bash
# Test shell for Reckon.
echo "Hello World"

while true; do
  echo -n "! "
  read cmd
  case $cmd in
    test) echo "This is a test script";;
    nope) sleep 2; echo "Wait what happened?";;
    "commit synchronize") echo "no route to re1";;
    quit) exit;;
  esac
done
