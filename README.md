# Poker

Play poker online poker without third parties (and without fees). Bet with NEAR. **Profit**.
Based on [Mental Poker](https://en.wikipedia.org/wiki/Mental_poker) algorithm [proposed by Shamir, Rivest, Adleman](https://apps.dtic.mil/dtic/tr/fulltext/u2/a066331.pdf).

## Security details

The `poker` contract is a "fair game" with some caveats.

**Pros:**

- Unbiased deck shuffling.
- Provable and secret card drawing.

**Cons:**

- If a player leaves the game the game stalls. Tokens of the player that left the game are slashed and remaining tokens are turned back to other participants.
- ZK-Proofs are required to avoid:
  - players submitting invalid data while shuffling deck and partially revealing a card.
  - players learning secret information from other players.

## Setup

1. `npm install -g near-shell`

2. `pip install -r client/requirements.txt`

3. Login with wallet. Make sure you have access to you keys, usually they are stored on `neardev/**/<account_id>.json`

4. Launch the python client: `python3 client /path/to/account_id_key.json`

Once you start using the python client watch `poker.log` to see all the interactions taking place with the blockchain.

## How to play

This is how a games look using the client. On start type `help` as suggested to see all commands:

```
[bob]>>> help
Available commands:
[d]deck_state   Show raw deck state.
                args: <room_id>

[e]enter        Enter a room. Can only enter to play in rooms that are Initiating.
                args: <room_id>

[f]fold         Fold your cards for this round.
                args: <room_id>

[h]help         Show this help
                args:

[l]list         List all rooms.
                args:

[n]new_room     Create a new room.
                args: <name>

[p]poker_state  Show raw poker table state.
                args: <room_id>

[r]raise        Increase your current bet TO amount.
                args: <amount> <room_id>

[s]start        Start the game in a room if it is Initiating or Idle
                args: <room_id>

[t]state        Show game state.
                args: <room_id>
```

### Create a new room

```
[bob]>>> new_room poker_arena
Created room poker_arena with id 1
```

### List all rooms

```
[bob]>>> list
000 qwerty Idle
001 poker_arena Initiating
```

Each row of the output describes a room: `id name current_status`
You can only enter in rooms with state: `Initiating`

### Enter a room

```
[bob]>>> enter 1


[bob]>>>
  Name  | Cards | Total | Staked | On Game | Turn
--------+-------+-------+--------+---------+------
 bob(*) |       |  1000 |   0    |   True  |

Status: Initiating
```

Notice that the id is an integer. So no need to provide leading 0.
Right now fake tokens are used and all player start with 1000 tokens.

### Start the game

Wait for other players join the game. Board is updated automatically as new players join:

```
  Name  | Cards | Total | Staked | On Game | Turn
--------+-------+-------+--------+---------+------
 bob(*) |       |  1000 |   0    |   True  |
--------+-------+-------+--------+---------+------
 alice  |       |  1000 |   0    |   True  |
--------+-------+-------+--------+---------+------
 carol  |       |  1000 |   0    |   True  |

Status: Initiating
```

After entering a room you don't need to provide room argument necessarily for subsequent commands. Last room entered will be used by default.

Start the game writing `start`. You should see something similar to this:

```
[bob]>>>
  Name  | Cards | Total | Staked | On Game |    Turn
--------+-------+-------+--------+---------+-----------
 bob(*) |       |  1000 |   0    |   True  |
--------+-------+-------+--------+---------+-----------
 alice  |       |  1000 |   0    |   True  | Shuffling
--------+-------+-------+--------+---------+-----------
 carol  |       |  1000 |   0    |   True  |

[bob]>>>
  Name  | Cards | Total | Staked | On Game |    Turn
--------+-------+-------+--------+---------+-----------
 bob(*) |       |  1000 |   0    |   True  |
--------+-------+-------+--------+---------+-----------
 alice  |       |  1000 |   0    |   True  |
--------+-------+-------+--------+---------+-----------
 carol  |       |  1000 |   0    |   True  | Shuffling

[bob]>>>
  Name  | Cards | Total | Staked | On Game |    Turn
--------+-------+-------+--------+---------+-----------
 bob(*) |       |  1000 |   0    |   True  |
--------+-------+-------+--------+---------+-----------
 alice  |       |  1000 |   0    |   True  | Shuffling
--------+-------+-------+--------+---------+-----------
 carol  |       |  1000 |   0    |   True  |

[bob]>>>
  Name  | Cards | Total | Staked | On Game |    Turn
--------+-------+-------+--------+---------+-----------
 bob(*) |       |  1000 |   6    |   True  |
--------+-------+-------+--------+---------+-----------
 alice  |       |  1000 |   0    |   True  | Revealing
--------+-------+-------+--------+---------+-----------
 carol  |       |  1000 |   3    |   True  |

[bob]>>>
```

The board is being updated while state is changing. Initially all players need to shuffle and encrypt the deck (this is done automatically but requires some time). After deck is shuffled initial cards are dealt to participants.

```
[bob]>>>
  Name  | Cards  | Total | Staked | On Game |   Turn
--------+--------+-------+--------+---------+---------
 bob(*) | 10♠ 9♦ |  1000 |   6    |   True  |
--------+--------+-------+--------+---------+---------
 alice  |        |  1000 |   0    |   True  | Betting
--------+--------+-------+--------+---------+---------
 carol  |        |  1000 |   3    |   True  |

[bob]>>>
```

For example carol will be seeing a different board:

```
[carol]>>>
   Name   | Cards | Total | Staked | On Game |   Turn
----------+-------+-------+--------+---------+---------
   bob    |       |  1000 |   6    |   True  |
----------+-------+-------+--------+---------+---------
  alice   |       |  1000 |   0    |   True  | Betting
----------+-------+-------+--------+---------+---------
 carol(*) | 8♦ 9♥ |  1000 |   3    |   True  |

[carol]>>>
```

And alice:

```
[alice]>>>
   Name   | Cards | Total | Staked | On Game |   Turn
----------+-------+-------+--------+---------+---------
   bob    |       |  1000 |   6    |   True  |
----------+-------+-------+--------+---------+---------
 alice(*) | 3♠ 4♦ |  1000 |   0    |   True  | Betting
----------+-------+-------+--------+---------+---------
  carol   |       |  1000 |   3    |   True  |

[alice]>>>
```

Staked column denotes how much is at stake at this moment by every participant. Initially there is a big blind of 6 token by player at seat 1, and small blind by previous player (at seat n). Player next to the big blind is first to play, in this case alice at seat 2.

Turn column is denotes which player should play and what is the expected type of action from it.
User interaction in the middle of a round is only required on state `Betting`.

For the purpose of demonstration we will show the point of view of each player. You can notice which player we are referring to from the context of from the name in the prompt.

### Betting

When is the turn to bet for a player it has two options: (`Fold` and `Raise`).

**Folding**

Alice will fold as she has very bad hand and nothing at stake.

```
[alice]>>> fold
{'Ok': None}

[alice]>>>
   Name   | Cards | Total | Staked | On Game |   Turn
----------+-------+-------+--------+---------+---------
   bob    |       |  1000 |   6    |   True  |
----------+-------+-------+--------+---------+---------
 alice(*) | 3♠ 4♦ |  1000 |   0    |  False  |
----------+-------+-------+--------+---------+---------
  carol   |       |  1000 |   3    |   True  | Betting
```

After alice fold its Carol turn. The column *On Game* denotes player that haven't fold so far. Since Alice just fold it is not longer playing this hand.

**Raise**

When typing `raise amount` it imply you will increase your stake to `amount` (not adding). It is not valid to raise less than max stake or more than total amount of token. There is a particular case when it is allowed to raise less than max stake and it is when is raised to total token (*All-in*).

Carol will raise the bet to 10 using:

```
[carol]>>> raise 10
{'Ok': None}

[carol]>>>
   Name   | Cards | Total | Staked | On Game |   Turn
----------+-------+-------+--------+---------+---------
   bob    |       |  1000 |   6    |   True  | Betting
----------+-------+-------+--------+---------+---------
  alice   |       |  1000 |   0    |  False  |
----------+-------+-------+--------+---------+---------
 carol(*) | 8♦ 9♥ |  1000 |   10   |   True  |

[carol]>>>
```

**Calling**

And Bob will see Carol bet.

```
[bob]>>> raise 10
{'Ok': None}

...

[bob]>>>
  Name  | Cards  | Total | Staked | On Game |   Turn
--------+--------+-------+--------+---------+---------
 bob(*) | 10♠ 9♦ |  1000 |   10   |   True  |
--------+--------+-------+--------+---------+---------
 alice  |        |  1000 |   0    |  False  |
--------+--------+-------+--------+---------+---------
 carol  |        |  1000 |   10   |   True  | Betting

Table: 9♣ 4♥ 8♥
```

A lot of boards will be displayed before this last board, since to reveal each card all participants needs to interact with the blockchain which might take some time.

Notice in the bottom of the board the three cards revealed on the *Flop*.

## Summary

Up to this point you have the basics about how to interact with this tool. Notice that rounds might take long time since it requires communication with the blockchain sequentially (not in parallel) from all players.

## Roadmap

1. Determine round winners and give pot back to them.
2. Use NEAR tokens.
3. Player who loose al its cash should be removed from the game.
4. Slash participant that stalls the game and recover from that state.
5. Add ZK-Proof to avoid invalid data while interacting with the deck.
6. Improve communication performance.
