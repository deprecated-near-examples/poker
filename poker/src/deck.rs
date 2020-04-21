use crate::types::AccountId;
use crate::types::CryptoHash;
use crate::types::{CardId, PlayerId};
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::env;
use serde::Serialize;

#[derive(Serialize, BorshDeserialize, BorshSerialize, Debug)]
pub enum DeckError {
    DeckAlreadyInitiated,
    DeckNotInShufflingState,
    NotPossibleToStartReveal,
    PlayerAlreadyInGame,
    PlayerNotInGame,
    InvalidTurn,
    InvalidPlayerId,
    InvalidCardId,
    /// Tried to fetch revealed card, but it is not revealed yet.
    CardNotRevealed,
    /// Tried to reveal part but not in revealing state
    NotRevealing,
    /// Tried to reveal part but it's not player turn to reveal
    PlayerCantReveal,
}

#[derive(PartialEq, Eq, Clone, BorshDeserialize, BorshSerialize, Serialize, Debug)]
pub enum DeckStatus {
    Initiating,
    Shuffling(PlayerId),
    Running,
    /// Revealing progress is ongoing
    Revealing {
        // Card to be revealed
        card_id: CardId,
        // Player to whom this card will be revealed.
        // None if it is going to be revealed to all players.
        receiver: Option<PlayerId>,
        // Player that should submit its part in this turn.
        // It can be the receiver if it should fetch its part.
        turn: PlayerId,
        // Partially decrypted card.
        progress: CryptoHash,
    },
    Closed,
}

impl Default for DeckStatus {
    fn default() -> Self {
        Self::Initiating
    }
}

#[derive(BorshDeserialize, BorshSerialize, Default, Serialize, Clone)]
pub struct Deck {
    status: DeckStatus,
    players: Vec<AccountId>,
    cards: Vec<CryptoHash>,
    pub revealed: Vec<Option<CryptoHash>>,
}

impl Deck {
    // TODO: Add password.
    // TODO: Add minimum/maximum amount of players.
    pub fn new(num_cards: u64) -> Deck {
        Deck {
            status: DeckStatus::Initiating,
            players: vec![],
            cards: (0..num_cards).map(|num| num.to_string()).collect(),
            revealed: vec![None; num_cards as usize],
        }
    }

    pub fn get_players(&self) -> Vec<AccountId> {
        self.players.clone()
    }

    pub fn num_players(&self) -> u64 {
        self.players.len() as u64
    }

    pub fn get_player_id(&self) -> Result<PlayerId, DeckError> {
        let account_id = env::signer_account_id();
        self.players
            .iter()
            .position(|player_account_id| player_account_id == &account_id)
            .map(|pos| pos as PlayerId)
            .ok_or(DeckError::PlayerNotInGame)
    }

    pub fn enter(&mut self) -> Result<(), DeckError> {
        if self.status == DeckStatus::Initiating {
            let account_id = env::signer_account_id();
            if self.players.contains(&account_id) {
                Err(DeckError::PlayerAlreadyInGame)
            } else {
                self.players.push(account_id);
                Ok(())
            }
        } else {
            Err(DeckError::DeckAlreadyInitiated)
        }
    }

    pub fn start(&mut self) -> Result<(), DeckError> {
        if self.status != DeckStatus::Initiating {
            Err(DeckError::DeckAlreadyInitiated)
        } else {
            self.status = DeckStatus::Shuffling(0);
            Ok(())
        }
    }

    pub fn get_status(&self) -> DeckStatus {
        self.status.clone()
    }

    pub fn get_turn(&self) -> Option<PlayerId> {
        match self.status {
            DeckStatus::Closed | DeckStatus::Running | DeckStatus::Initiating => None,
            DeckStatus::Shuffling(player_id) => Some(player_id),
            DeckStatus::Revealing { turn, .. } => Some(turn),
        }
    }

    pub fn get_revealed_card(&self, card_id: CardId) -> Result<CryptoHash, DeckError> {
        self.revealed
            .get(card_id as usize)
            .ok_or(DeckError::InvalidCardId)?
            .clone()
            .ok_or(DeckError::CardNotRevealed)
    }

    pub fn get_partial_shuffle(&self) -> Result<Vec<CryptoHash>, DeckError> {
        if let DeckStatus::Shuffling(_) = self.status {
            Ok(self.cards.clone())
        } else {
            Err(DeckError::DeckNotInShufflingState)
        }
    }

    pub fn close(&mut self) {
        self.status = DeckStatus::Closed;
    }

    // TODO: Add zk-proof for correct computation of the cards (given previous set of cards and public key)
    pub fn submit_shuffled(&mut self, new_cards: Vec<CryptoHash>) -> Result<(), DeckError> {
        if let DeckStatus::Shuffling(current_player_id) = self.status {
            let player_id = self.get_player_id()?;
            if player_id != current_player_id {
                Err(DeckError::InvalidTurn)
            } else {
                self.cards = new_cards;

                if current_player_id + 1 < self.num_players() {
                    self.status = DeckStatus::Shuffling(current_player_id + 1);
                } else {
                    self.status = DeckStatus::Running;
                }

                Ok(())
            }
        } else {
            Err(DeckError::DeckNotInShufflingState)
        }
    }

    /// Reveal card at position `card_id` to player at position `player_id`.
    /// If `player_id` is None, reveal the card to all
    pub fn reveal_card(
        &mut self,
        card_id: u64,
        receiver_player_id: Option<PlayerId>,
    ) -> Result<(), DeckError> {
        if self.status == DeckStatus::Running {
            if card_id as usize >= self.cards.len() {
                return Err(DeckError::InvalidCardId);
            }

            if let Some(receiver_player_id) = receiver_player_id {
                let num_players = self.num_players();

                if receiver_player_id >= num_players {
                    return Err(DeckError::InvalidPlayerId);
                }

                let turn = if num_players == 1 {
                    0
                } else if receiver_player_id == 0 {
                    1
                } else {
                    0
                };

                self.status = DeckStatus::Revealing {
                    card_id,
                    receiver: Some(receiver_player_id),
                    turn,
                    progress: self.cards[card_id as usize].clone(),
                };

                Ok(())
            } else {
                self.status = DeckStatus::Revealing {
                    card_id,
                    receiver: None,
                    turn: 0,
                    progress: self.cards[card_id as usize].clone(),
                };
                Ok(())
            }
        } else {
            Err(DeckError::NotPossibleToStartReveal)
        }
    }

    // TODO: Add zk-proof using previous part and public key
    pub fn submit_reveal_part(&mut self, card: CryptoHash) -> Result<(), DeckError> {
        if let DeckStatus::Revealing {
            card_id,
            receiver,
            turn,
            progress: _,
        } = self.status.clone()
        {
            let player_id = self.get_player_id()?;

            if player_id != turn || receiver.map_or(false, |receiver| player_id == receiver) {
                return Err(DeckError::PlayerCantReveal);
            }

            let mut next_turn = turn + 1;

            if let Some(receiver) = receiver {
                if receiver == next_turn {
                    next_turn += 1;
                }
            }

            if next_turn == self.num_players() {
                if receiver.is_some() {
                    self.status = DeckStatus::Revealing {
                        card_id,
                        receiver,
                        turn: receiver.unwrap(),
                        progress: card,
                    }
                } else {
                    self.revealed[card_id as usize] = Some(card);
                    self.status = DeckStatus::Running;
                }
            } else {
                self.status = DeckStatus::Revealing {
                    card_id,
                    receiver,
                    turn: next_turn,
                    progress: card,
                };
            }

            Ok(())
        } else {
            Err(DeckError::NotRevealing)
        }
    }

    /// Receiver of the revealing card should call this function after downloading
    /// partially encrypted card to finish the revealing process.
    pub fn finish_reveal(&mut self) -> Result<(), DeckError> {
        if let DeckStatus::Revealing {
            card_id: _,
            receiver,
            turn,
            progress: _,
        } = self.status.clone()
        {
            let player_id = self.get_player_id()?;

            if let Some(receiver) = receiver {
                if receiver == player_id && turn == player_id {
                    self.status = DeckStatus::Running;
                    return Ok(());
                }
            }
            Err(DeckError::PlayerCantReveal)
        } else {
            Err(DeckError::NotRevealing)
        }
    }
}
