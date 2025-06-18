# Playspawn

[![Web Client](https://github.com/nabware/playspawn/actions/workflows/web-client.yaml/badge.svg)](https://github.com/nabware/playspawn/actions/workflows/web-client.yaml)
[![Web API](https://github.com/nabware/playspawn/actions/workflows/web-api.yaml/badge.svg)](https://github.com/nabware/playspawn/actions/workflows/web-api.yaml)
[![Game API](https://github.com/nabware/playspawn/actions/workflows/game-api.yaml/badge.svg)](https://github.com/nabware/playspawn/actions/workflows/game-api.yaml)

## Architecture

### Autoscaling virtual machines

- Web client
- Web API
- Game API

### Managed services

- SQL database
- In-memory database

## Data models

### ServiceHealthLog

- id
- service_name
- target_request_latency
- average_request_latency
- requests_per_second
- current_instance_count
- created_at
- updated_at

### ServiceScalingEvent

- id
- service_name
- requested_instance_delta
- started_at
- ended_at
- created_at
- updated_at

### InstanceHealthLog

- id
- instance_id
- instance_type
- service_name
- average_request_latency
- requests_per_second
- requests_in_progress
- current_connection_count
- cpu_util
- ram_used
- ram_total
- network_in
- network_out
- disk_read
- disk_write
- disk_used
- disk_total
- created_at
- updated_at

### ServiceRequestLog

- id
- service_name
- started_at
- ended_at
- latency
- protocol
- method
- path
- type
- status_code
- context
- instance_id
- user_id
- created_at
- updated_at

### User

- id
- email
- email_verified
- password
- given_name
- family_name
- display_name
- created_at
- updated_at

### EmailVerification

- id
- user_id
- token
- sent_to_email
- expires_at
- verified_at
- created_at
- updated_at

### IdentityProvider

- id
- name
- created_at
- updated_at

### IdentityProviderConnection

- id
- provider_id
- provider_user_id
- user_id
- created_at
- updated_at

### Game

- id
- name
- created_at
- updated_at

### GameSession

- id
- game_id
- created_at
- updated_at

### GameSessionMember

- id
- session_id
- user_id
- is_winner
- created_at
- updated_at
