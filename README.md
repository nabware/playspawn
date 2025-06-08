# Playspawn

## Architecture

### Autoscaling virtual machines

- Web client
- API server
- WebSocket server

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

### Group

- id
- owner_user_id
- name
- created_at
- updated_at

### GroupMembership

- id
- group_id
- user_id
- created_at
- updated_at

### GroupInvitation

- id
- group_id
- token
- expires_at
- created_at
- updated_at

### GroupMessage

- id
- group_id
- user_id
- content
- created_at
- updated_at
