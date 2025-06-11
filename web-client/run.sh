#!/bin/bash

MODE=$1

# Start the Web API server
echo "Starting Web API server..."
cd ../web-api
cargo run &
WEB_API_PID=$!

# Wait for the Web API to be ready
echo "Waiting for Web API to be ready..."
until curl -s http://localhost:8080 >/dev/null; do
  sleep 1
done

case $MODE in
dev | "")
  echo "Running dev server..."
  cd ../web-client
  npm run dev
  ;;
test)
  echo "Running tests..."
  cd ../web-client
  npx playwright test
  ;;
*)
  echo "Invalid mode: '$MODE'"
  ;;
esac

echo "Stopping Web API..."
kill $WEB_API_PID
