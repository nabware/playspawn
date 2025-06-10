#!/bin/bash

echo "Starting Web API server..."
cd ../web-api
cargo run & 
WEB_API_PID=$!

echo "Waiting for Web API to be ready..."
cd ../web-client
until curl -s http://localhost:8080 > /dev/null; do
  sleep 1
done

echo "Web API is up. Running Playwright tests..."
npx playwright test

# Kill the API after tests
kill $WEB_API_PID
