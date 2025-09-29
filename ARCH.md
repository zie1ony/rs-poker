# Architecture of running a Poker Tournament Series

A Series consist of N Tournaments of the same type with the same players.

## Series

Series entity is responsible for spawning N tournaments with different
players order. It collects results of each tournament and aggregates them.

## Torunament

Tournament is a game after game played, until the end condition is met.

### Settings
- `see_historical_thoughts`: bool - if true, players can see their own past
  chain of thoughts for each action they made.
- `public_chat`: bool - if true, players can say something publicly to other
  players on every action.
- `players`: Vec<Player> - list of players in the tournament.
- `end_condition`: - SingleWinner | MaxGames(N) - condition to end the tournament.

