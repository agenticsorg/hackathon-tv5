#!/bin/bash

# Kill processes running on ports 3000–3006

for port in {3000..3006}
do
  pid=$(lsof -t -i tcp:$port)

  if [ -n "$pid" ]; then
    echo "Killing process on port $port (PID: $pid)"
    kill -9 $pid
  else
    echo "No process found on port $port"
  fi
done

echo "Done."
