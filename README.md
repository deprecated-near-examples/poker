# Poker

## Deck automata description

Status: Initiating

start should be called from the Poker contract.
only one player registered can do it.

Status: Shuffling(i)

    Player i should call:
        call get_partial_shuffle
        encrypt_cards
        shuffle_cards
        call submit_shuffled

Status: Running

Status: Revealing
    Player which is its turn:
        call get_status
        decrypt card

        if this player is target:
            call finish_reveal
        else:
            call submit_reveal_part

Eventually the deck will be closed from the Poker struct.

## Poker automata description

Players enters in the game.

Status: Initiating (same as deck). | Idle

start is called by one of the players in the game.
=> DeckAction(Shuffling)

Reveal cards for each player.

BettingFirstRound

## Poker description

Start several rounds.
1. Any player can close at the end of each round.
2. New room is needed if some player leaves.
3. Button still a

+ Dealing cards
    + Betting round
+ Show 3 cards
    + Betting round
+ Show one card
    + Betting round
+ Show one card
    + Betting round
+ Reveal cards of standing player.
+ Assign resources to winners.
    Fast hack: win higher card
    Fast hack: player who wins earns it all.
        In practice this should be modified to give away money properly.

Betting round:
    Submit(amount).
        amount should be less or equal than you have.
        if amount < than you have then:
            amount >= max_raised
    Check()
    Fold()

    If the someone has raise before you then you can call or raise

## TODO | Client

- Game context

## TODO | Contract

- Only registered players can start the game
- Only registered players can close the game on rounds ending.
- Use NEAR Tokens
- Reveal several cards at the same time.