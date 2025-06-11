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
