use crate::deck::{Deck, DeckError, DeckStatus};
use crate::poker::{ActionResponse, BetAction, Poker, PokerError, PokerStatus};
use crate::types::{CryptoHash, PlayerId, RoomId};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug)]
pub enum GameError {
    RoomIdNotFound,
    OngoingRound,
    DeckError(DeckError),
    PokerError(PokerError),
}

impl From<DeckError> for GameError {
    fn from(deck_error: DeckError) -> Self {
        GameError::DeckError(deck_error)
    }
}

impl From<PokerError> for GameError {
    fn from(poker_error: PokerError) -> Self {
        GameError::PokerError(poker_error)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Eq, PartialEq, Clone)]
pub enum GameStatus {
    // Start haven't been called. Players are able to enter the game.
    Initiating,
    // Round already finished. Call start to start next round.
    Idle,
    // Need action by some player in deck.
    DeckAction,
    // Need action by some player in poker.
    PokerAction,
    // Game have been closed
    Closed,
}

impl GameStatus {
    pub fn is_active(&self) -> bool {
        *self != GameStatus::Closed
    }

    pub fn is_initiating(&self) -> bool {
        *self == GameStatus::Initiating
    }
}

// TODO: Use NEAR Tokens
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Game {
    pub name: String,
    pub id: RoomId,
    pub status: GameStatus,
    deck: Deck,
    poker: Poker,
}

impl Game {
    pub fn new(name: String, id: RoomId) -> Self {
        Self {
            name,
            id,
            status: GameStatus::Initiating,
            deck: Deck::new(52),
            poker: Poker::new(),
        }
    }

    pub fn enter(&mut self) -> Result<(), GameError> {
        self.deck.enter().map_err(Into::<GameError>::into)?;
        // TODO: Put min tokens / max tokens caps
        self.poker.new_player(1000);
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), GameError> {
        match self.status {
            GameStatus::Initiating | GameStatus::Idle => {
                self.deck.start().map_err(Into::<GameError>::into)?;
                self.status = GameStatus::DeckAction;
                Ok(())
            }
            _ => Err(GameError::OngoingRound),
        }
    }

    pub fn close(&mut self) -> Result<(), GameError> {
        match self.status {
            GameStatus::Initiating | GameStatus::Idle => {
                self.deck.close();
                self.status = GameStatus::Closed;
                Ok(())
            }
            _ => Err(GameError::OngoingRound),
        }
    }

    fn check_status(&mut self) {
        self.status = match self.poker.status {
            PokerStatus::Idle => GameStatus::Idle,
            PokerStatus::Dealing {
                player_id, card_id, ..
            } => {
                self.deck.reveal_card(card_id, Some(player_id)).expect(
                    format!(
                        "Impossible to reveal card {} for player {}",
                        card_id, player_id
                    )
                    .as_ref(),
                );

                GameStatus::DeckAction
            }
            PokerStatus::Betting { .. } => GameStatus::PokerAction,
            PokerStatus::Revealing { card_id, .. } | PokerStatus::Showdown { card_id, .. } => {
                self.deck.reveal_card(card_id, None).expect(
                    format!("Impossible to reveal card {} for the table.", card_id).as_ref(),
                );
                GameStatus::DeckAction
            }
            PokerStatus::WaitingRevealedCards => {
                self.poker.submit_revealed_cards(self.deck.revealed.clone());
                GameStatus::Idle
            }
        };
    }

    /// Deck finalized one step.
    fn check_next_status(&mut self) {
        let deck_status = self.deck.get_status();

        if deck_status != DeckStatus::Running {
            self.status = GameStatus::DeckAction;
            return;
        }

        self.poker.next();
        self.check_status();
    }

    pub fn deck_state(&self) -> Deck {
        self.deck.clone()
    }

    pub fn poker_state(&self) -> Poker {
        self.poker.clone()
    }

    pub fn state(&self) -> GameStatus {
        self.status.clone()
    }

    pub fn player_id(&self) -> Result<PlayerId, GameError> {
        self.deck.get_player_id().map_err(Into::into)
    }

    // TODO: Implement this method to find guilty that stalled the game.
    // /// Current player that should make an action.
    // pub fn required_action(&self) -> Option<PlayerId> {}

    // TODO: Mechanism to slash participants that are stalling the game
    //       Discussion: Using some number of epochs, elapsed without inactivity.
}

// Implement Deck public interface for Game
impl Game {
    pub fn get_partial_shuffle(&self) -> Result<Vec<CryptoHash>, GameError> {
        self.deck.get_partial_shuffle().map_err(Into::into)
    }

    pub fn submit_shuffled(&mut self, new_cards: Vec<CryptoHash>) -> Result<(), GameError> {
        self.deck
            .submit_shuffled(new_cards)
            .map_err(Into::<GameError>::into)?;

        self.check_next_status();
        Ok(())
    }

    pub fn finish_reveal(&mut self) -> Result<(), GameError> {
        self.deck.finish_reveal().map_err(Into::<GameError>::into)?;

        self.check_next_status();
        Ok(())
    }

    pub fn submit_reveal_part(&mut self, card: CryptoHash) -> Result<(), GameError> {
        self.deck
            .submit_reveal_part(card)
            .map_err(Into::<GameError>::into)?;

        self.check_next_status();
        Ok(())
    }
}

// TODO: On Initiating/Idle games allow leaving and claiming back all winned tokens.
// Implement Poker public interface for Game
impl Game {
    pub fn submit_bet_action(&mut self, bet: BetAction) -> Result<(), GameError> {
        self.poker
            .submit_bet_action(ActionResponse {
                player_id: self.player_id()?,
                action: bet,
            })
            .map_err(Into::<GameError>::into)?;

        self.check_status();
        Ok(())
    }
}
