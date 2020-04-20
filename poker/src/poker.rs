use crate::types::CardId;
use crate::types::CryptoHash;
use crate::types::PlayerId;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone, PartialEq, Eq)]
pub enum Stage {
    Flop,
    Turn,
    River,
    Showdown,
}

impl Stage {
    fn next(&self) -> Self {
        match self {
            Stage::Flop => Stage::Turn,
            Stage::Turn => Stage::River,
            Stage::River => Stage::Showdown,
            Stage::Showdown => panic!("No next stage after showdown"),
        }
    }

    fn cards_to_reveal(&self) -> u8 {
        match self {
            Stage::Flop => 3,
            Stage::Turn => 1,
            Stage::River => 1,
            Stage::Showdown => panic!("No cards to reveal at showdown"),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone, Eq, PartialEq)]
pub enum PokerStatus {
    Idle,
    Dealing {
        player_id: PlayerId,
        card_id: CardId,
        first_card: bool,
    },
    Betting {
        // Waiting for player `target` to make an action.
        target: PlayerId,
        // Last player to raise. If raised is false this is big blind.
        until: PlayerId,
        // If some player have raised in this round.
        raised: bool,
        // Current max stake that must be called or raised
        max_stake: u64,
        // Next stage to play.
        next_stage: Stage,
    },
    Revealing {
        stage: Stage,
        card_id: CardId,
        missing_to_reveal: u8,
    },
    Showdown {
        player_id: PlayerId,
        card_id: CardId,
        first_card: bool,
    },
    WaitingRevealedCards,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug)]
pub enum PokerError {
    InvalidPlayerId,

    TooLowStake,
    NotEnoughStake,

    NotBettingRound,
    NotBettingTurn,
}

#[derive(Serialize, Deserialize)]
pub enum BetAction {
    Fold,
    Stake(u64),
}

pub struct ActionResponse {
    pub player_id: PlayerId,
    pub action: BetAction,
}

// TODO: Remove automatically from the game players with 0 tokens.
//       Mainly regarding card signatures.

/// Raw poker implementation.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
pub struct Poker {
    /// Number of tokens each player has available.
    tokens: Vec<u64>,
    /// Currently staked tokens.
    staked: Vec<u64>,
    /// Players that have already folded its cards in this turn.
    folded: Vec<bool>,
    /// Current status.
    pub status: PokerStatus,
    /// Number of token used for the big blind. This value will double on each game.
    blind_token: u64,
    /// Player which is the big blind on next round.
    big_blind: PlayerId,
    /// Card on the top of the stack.
    first_unrevealed_card: CardId,
}

impl Poker {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            staked: vec![],
            folded: vec![],
            status: PokerStatus::Idle,
            blind_token: 6,
            big_blind: 0,
            first_unrevealed_card: 0,
        }
    }

    pub fn new_player(&mut self, tokens: u64) {
        self.tokens.push(tokens);
        self.staked.push(0);
        self.folded.push(false);
    }

    fn prev_player(&self, player_id: PlayerId) -> PlayerId {
        if player_id == 0 {
            self.num_players() - 1
        } else {
            player_id - 1
        }
    }

    fn next_player(&self, player_id: PlayerId) -> PlayerId {
        if player_id + 1 == self.num_players() {
            0
        } else {
            player_id + 1
        }
    }

    fn next_on_game(&self, mut player_id: PlayerId) -> PlayerId {
        for _ in 0..self.num_players() {
            if !self.folded[player_id as usize] {
                return player_id;
            } else {
                player_id = self.next_player(player_id);
            }
        }
        panic!("All players folded.");
    }

    /// Number of players who have already folded
    fn total_folded(&self) -> u64 {
        self.folded.iter().filter(|&folded| *folded).count() as u64
    }

    /// Increase the stake for player_id to stake. If stake is less than current staked
    /// by this player, it will return Error without changing anything.
    /// If stake is bigger than total tokens, it will stake all tokens.
    fn try_stake(&mut self, player_id: PlayerId, stake: u64) -> Result<(), PokerError> {
        let total = self
            .tokens
            .get(player_id as usize)
            .ok_or(PokerError::InvalidPlayerId)?;

        if stake < self.staked[player_id as usize] {
            return Err(PokerError::TooLowStake);
        }

        self.staked[player_id as usize] = std::cmp::min(stake, *total);
        Ok(())
    }

    pub fn get_status(&self) -> PokerStatus {
        self.status.clone()
    }

    /// Get topmost card not used yet. Mark this card as used.
    fn get_card(&mut self) -> CardId {
        self.first_unrevealed_card += 1;
        self.first_unrevealed_card - 1
    }

    fn card_id_from_player(&self, player_id: PlayerId, first_card: bool) -> CardId {
        2 * player_id + (!first_card as u64)
    }

    pub fn next(&mut self) {
        match self.status.clone() {
            PokerStatus::Idle => {
                // Make small blind and big blinds bet
                self.try_stake(self.big_blind, self.blind_token).unwrap();
                self.try_stake(self.prev_player(self.big_blind), self.blind_token / 2)
                    .unwrap();

                self.status = PokerStatus::Dealing {
                    player_id: 0,
                    card_id: self.get_card(),
                    first_card: true,
                };
            }
            PokerStatus::Dealing {
                player_id,
                first_card,
                ..
            } => {
                if first_card {
                    self.status = PokerStatus::Dealing {
                        player_id,
                        card_id: self.get_card(),
                        first_card: false,
                    };
                } else {
                    if player_id + 1 == self.num_players() {
                        // All cards where already dealt. Start first round of betting.
                        let target = self.next_player(self.big_blind);
                        self.status = PokerStatus::Betting {
                            target,
                            until: self.big_blind,
                            raised: false,
                            max_stake: self.blind_token,
                            next_stage: Stage::Flop,
                        };
                    } else {
                        self.status = PokerStatus::Dealing {
                            player_id: player_id + 1,
                            card_id: self.get_card(),
                            first_card: true,
                        };
                    }
                }
            }
            PokerStatus::Revealing {
                stage,
                missing_to_reveal,
                ..
            } => {
                if missing_to_reveal == 0 {
                    // Find next player who have not folded after the big blind.
                    let target = self.next_on_game(self.next_player(self.big_blind));
                    self.status = PokerStatus::Betting {
                        target,
                        until: self.big_blind,
                        raised: false,
                        max_stake: self.blind_token,
                        next_stage: stage.next(),
                    };
                } else {
                    self.status = PokerStatus::Revealing {
                        stage,
                        card_id: self.get_card(),
                        missing_to_reveal: missing_to_reveal - 1,
                    };
                }
            }
            PokerStatus::Showdown {
                player_id,
                first_card,
                ..
            } => {
                if first_card {
                    self.status = PokerStatus::Showdown {
                        player_id,
                        card_id: self.card_id_from_player(player_id, false),
                        first_card: false,
                    };
                } else {
                    let next_player = self.next_on_game(player_id);

                    if next_player < player_id {
                        // All cards were revealed
                        self.status = PokerStatus::WaitingRevealedCards;
                    } else {
                        self.status = PokerStatus::Showdown {
                            player_id: next_player,
                            card_id: self.card_id_from_player(player_id, true),
                            first_card: true,
                        };
                    }
                }
            }
            PokerStatus::Betting { .. } => panic!("Called next on betting state."),
            PokerStatus::WaitingRevealedCards => {
                panic!("Called next while waiting for revealed cards.")
            }
        }
    }

    /// Call when the round is over. Update the state of the game for the next round.
    fn finish(&mut self, _winners: Vec<PlayerId>) {
        self.status = PokerStatus::Idle;
        self.big_blind = self.next_player(self.big_blind);
        self.blind_token *= 2;
        self.first_unrevealed_card = 0;

        // TODO: Reassign stake to winners.
        // TODO: Reset state
    }

    fn start_stage(&mut self, stage: Stage) {
        if stage == Stage::Showdown {
            let player_id = self.next_on_game(0);
            self.status = PokerStatus::Showdown {
                player_id,
                card_id: self.card_id_from_player(player_id, true),
                first_card: true,
            };
        } else {
            let missing_to_reveal = stage.cards_to_reveal() - 1;
            self.status = PokerStatus::Revealing {
                stage,
                card_id: self.get_card(),
                missing_to_reveal,
            };
        }
    }

    pub fn submit_revealed_cards(&mut self, _cards: Vec<Option<CryptoHash>>) {
        if self.status != PokerStatus::WaitingRevealedCards {
            panic!("Not waiting revealed cards");
        }

        // TODO: Find winners from cards revealed.

        self.finish(vec![]);

        todo!();
    }

    /// Submit the bet option from the player that is its turn.
    pub fn submit_bet_action(&mut self, action: ActionResponse) -> Result<(), PokerError> {
        match self.status.clone() {
            PokerStatus::Betting {
                target,
                until,
                raised,
                max_stake,
                next_stage,
            } => {
                if target != action.player_id {
                    return Err(PokerError::NotBettingTurn);
                }

                match action.action {
                    BetAction::Fold => {
                        self.folded[action.player_id as usize] = true;
                        let next_player = self.next_on_game(action.player_id);

                        if self.total_folded() + 1 == self.num_players() {
                            // All players but one have folded. That is the winner.
                            self.finish(vec![next_player]);
                            Ok(())
                        } else {
                            self.status = PokerStatus::Betting {
                                target: next_player,
                                until,
                                raised,
                                max_stake,
                                next_stage,
                            };
                            Ok(())
                        }
                    }
                    BetAction::Stake(stake) => {
                        if stake < max_stake {
                            Err(PokerError::TooLowStake)
                        } else if stake > self.tokens[action.player_id as usize] {
                            Err(PokerError::NotEnoughStake)
                        } else {
                            self.staked[action.player_id as usize] = stake;

                            if stake > max_stake {
                                // Raise
                                // TODO: Put a lower bound on raising
                                let next_player =
                                    self.next_on_game(self.next_player(action.player_id));

                                self.status = PokerStatus::Betting {
                                    target: next_player,
                                    until: next_player,
                                    raised: true,
                                    max_stake: stake,
                                    next_stage,
                                };
                                Ok(())
                            } else {
                                // Call
                                if action.player_id == until {
                                    // Finish betting round. All players call big blind bet.
                                    self.start_stage(next_stage);
                                    Ok(())
                                } else {
                                    let next_player =
                                        self.next_on_game(self.next_player(action.player_id));
                                    if next_player != until || !raised {
                                        // Missing some players to place their bets.
                                        self.status = PokerStatus::Betting {
                                            target: next_player,
                                            until,
                                            raised,
                                            max_stake,
                                            next_stage,
                                        };
                                        Ok(())
                                    } else {
                                        // Finish betting round. All players call last raised stake.
                                        self.start_stage(next_stage);
                                        Ok(())
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => Err(PokerError::NotBettingRound),
        }
    }

    fn num_players(&self) -> u64 {
        self.tokens.len() as u64
    }
}
