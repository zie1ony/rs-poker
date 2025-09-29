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

## Unify views
Both the game and the tournament views has `_full_view` and `_player_view`. Those could be unified.

## Autoplay when only one player is left

- No need to ask the player for action, when there is only one player left with chips.

```
Game Started - ID: GameId("game-wlyffz")
Small blind: 20
Big blind: 40
Players: 3
- Alice has stack: 260
- Bob has stack: 20
- Charlie has stack: 20

Hole Cards:
- Alice: Jd Jc
- Bob: 9s Tc
- Charlie: Kd Qc

Community Cards: Ad 5d Kc 2s 7s

--- Preflop ---
Bob posts small blind of 20 (stack: 0, pot: 20)
Charlie posts big blind of 40 (stack: 0, pot: 40)
Alice thinks: "With a pair of Jacks, I have a strong hand preflop. The minimum bet is 60, which puts additional pressure on my opponents and could potentially force them to fold. This aligns with my tight aggressive strategy."
Alice increases to 60 (stack: 200, pot: 100)

--- Flop ---
Flop cards: Ad 5d Kc
Alice thinks: "With a strong pair of Jacks and the current pot size, I need to bet aggressively to exert pressure on my opponents and protect my hand."
Alice increases to 100 (stack: 100, pot: 200)

--- Turn ---
Turn card: 2s
Alice thinks: "I have a strong hand with a pair of Jacks, and I want to build the pot and pressure my opponents to either fold or commit more chips."
Alice increases to 100 (stack: 0, pot: 300)

--- River ---
River card: 7s

--- Game Ended ---
Charlie wins 60 (stack after: 60)
Alice wins 240 (stack after: 240)
```

## Players with zero stack should be removed from the game.