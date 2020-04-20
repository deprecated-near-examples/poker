use crate::types::PlayerId;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub enum Stage {
    Flop,
    Turn,
    Reaver,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub enum PokerStatus {
    Dealing { player_id: PlayerId },
    Betting,
    Revealing { stage: Stage },
}

/// Reveal cards to each player
/// Accept bets
///
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Poker {
    /// Number of tokens each player has available
    tokens: Vec<u64>,
    /// Staked
    staked: Vec<u64>,
    status: PokerStatus,
}

impl Poker {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            staked: vec![],
            status: PokerStatus::Dealing { player_id: 0 },
        }
    }

    pub fn new_player(&mut self, tokens: u64) {
        self.tokens.push(tokens);
        self.staked.push(0);
    }

    // fn num_players(&self) -> u64 {
    //     self.tokens.len() as u64
    // }
}
