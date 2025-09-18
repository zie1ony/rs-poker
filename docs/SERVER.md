# Poker Server

Server lives in `src/server/` directory.
It doesn't need to support any authentication, as it is assumed to be behind a secure proxy.

## Game creation

Game can be created with `POST /game` endpoint.
User needs to specify:
- list of automated players (`random`, `all-in`, `calling`, `folding` (see
  `src/arena/agent/` for available agents)) and their names.
- list of outside players (usernames) and their names. Each outside player can
  also have a description. This is useful for displaying player info in the UI.
- initial stack sizes for all players (same for all players).
- small blind size (big blind is always 2x small blind).
- game_id (unique, optional, if not provided, a random one will be generated).
- game_description (optional).

The server will respond with `201 Created` and the game_id in the body.

## Game progress

Game progress can be queried with `GET /game/{game_id}` endpoint.
All automated players will play their turns automatically.
Outside players will have their turn marked as `waiting_for_action`.
To play their turn, user needs to call `POST /game/{game_id}/action`
with the action in the body.

Each action should also contain `reason` field, which is a string describing
the reason for the action that user can provide.
