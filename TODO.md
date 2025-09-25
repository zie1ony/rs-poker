# TODO

## Big blinds calculation problem

```
Game Started - ID: GameId("game-k2eji6")
Small blind: 40
Big blind: 80
Players: 3
- Alice (You) has stack: 42.291214
- Bob has stack: 0
- Charlie has stack: 257.7088

Your Hand: 3s 5h

--- Preflop ---
Charlie posts small blind of 40 (stack: 217.7088, pot: 40)
Alice posts big blind of 80 (stack: 0, pot: 82.291214)
```

## Showdown with cards.

Showdown doesn't include players' cards.
Show only cards that actally participated in the showdown.

### Control Tower

`run-cli control-tower -n 2` to run 2 Workers that can execute tournament.

# Main process
Starts the server and the ControlTower.
Is responsible for graceful shutdown of the server and the ControlTower, after Ctrl-C or if the ControlTower ends it's life.

# ControlTower process
- Talks to the server to get list of tournaments that must be finished.
- Starts Workers for a single tournament.
- Recieves notification from the Workers, that the tournament is finished. (can confirm with the server).
- If Worker process ends unexpectedly, it should stop all Workers and end itself.

# Worker process
- Executes a single tournament.
- 