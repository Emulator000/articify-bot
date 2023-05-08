#!/bin/bash

while true; do
  "./bin/articify-bot" >>./logs/articify.log

  echo "Process crashed with code $?. Restarting..." >&2

  sleep 1
done
