use crate::types::PlayerId;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;

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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
pub enum PokerStatus {
    Idle,
    Dealing {
        player_id: PlayerId,
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
        missing_to_reveal: u8,
    },
    Showdown {
        player_id: PlayerId,
        first_card: bool,
    },
    WaitingRevealedCards,
}

#[derive(Debug)]
pub enum PokerError {
    InvalidPlayerId,

    TooLowStake,
    NotEnoughStake,

    NotBettingRound,
    NotBettingTurn,
}

pub enum ActionRequest {
    /// Deal one card to a particular player or to the table.
    Deal(Option<PlayerId>),
    /// Waiting from the bet of one player.
    BetFrom(PlayerId),
    /// Reveal one card from one player.
    Reveal {
        player_id: PlayerId,
        first_card: bool,
    },
    /// Waiting for revealed cards.
    RevealedCards,
}

pub enum BetAction {
    Fold,
    Stake(u64),
}

pub struct ActionResponse {
    player_id: PlayerId,
    action: BetAction,
}

// TODO: Remove automatically from the game players with 0 tokens.
//       Mainly regarding card signatures.

/// Raw poker implementation.
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Poker {
    /// Number of tokens each player has available.
    tokens: Vec<u64>,
    /// Currently staked tokens.
    staked: Vec<u64>,
    /// Players that have already folded its cards in this turn.
    folded: Vec<bool>,
    /// Current status.
    status: PokerStatus,
    /// Number of token used for the big blind. This value will double on each game.
    blind_token: u64,
    /// Player which is the big blind on next round.
    big_blind: PlayerId,
}

// TODO: When game finishes multiply blind_token * 2
// TODO: When game finishes increase big_blind + 1 mod num_players

impl Poker {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            staked: vec![],
            folded: vec![],
            status: PokerStatus::Idle,
            blind_token: 6,
            big_blind: 0,
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

    pub fn next(&mut self) -> ActionRequest {
        match self.status.clone() {
            PokerStatus::Idle => {
                // Make small blind and big blinds bet
                self.try_stake(self.big_blind, self.blind_token).unwrap();
                self.try_stake(self.prev_player(self.big_blind), self.blind_token / 2)
                    .unwrap();

                self.status = PokerStatus::Dealing {
                    player_id: 0,
                    first_card: true,
                };
                ActionRequest::Deal(Some(0))
            }
            PokerStatus::Dealing {
                player_id,
                first_card,
            } => {
                if first_card {
                    self.status = PokerStatus::Dealing {
                        player_id,
                        first_card: false,
                    };
                    ActionRequest::Deal(Some(player_id))
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
                        ActionRequest::BetFrom(target)
                    } else {
                        self.status = PokerStatus::Dealing {
                            player_id: player_id + 1,
                            first_card: true,
                        };
                        ActionRequest::Deal(Some(player_id + 1))
                    }
                }
            }
            PokerStatus::Revealing {
                stage,
                missing_to_reveal,
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
                    ActionRequest::BetFrom(target)
                } else {
                    self.status = PokerStatus::Revealing {
                        stage,
                        missing_to_reveal: missing_to_reveal - 1,
                    };
                    ActionRequest::Deal(None)
                }
            }
            PokerStatus::Showdown {
                player_id,
                first_card,
            } => {
                if first_card {
                    self.status = PokerStatus::Showdown {
                        player_id,
                        first_card: false,
                    };
                    ActionRequest::Reveal {
                        player_id,
                        first_card: false,
                    }
                } else {
                    let next_player = self.next_on_game(player_id);

                    if next_player < player_id {
                        // All cards were revealed
                        self.status = PokerStatus::WaitingRevealedCards;
                        ActionRequest::RevealedCards
                    } else {
                        self.status = PokerStatus::Showdown {
                            player_id: next_player,
                            first_card: true,
                        };
                        ActionRequest::Reveal {
                            player_id: next_player,
                            first_card: true,
                        }
                    }
                }
            }
            PokerStatus::Betting { .. } => panic!("Called next on betting state."),
            PokerStatus::WaitingRevealedCards => {
                panic!("Called next while waiting for reveald cards.")
            }
        }
    }

    /// Finish
    fn finish(&mut self, _winners: Vec<PlayerId>) {
        self.status = PokerStatus::Idle;
        // TODO: Reassign stake to winners.
    }

    fn start_stage(&mut self, stage: Stage) -> ActionRequest {
        if stage == Stage::Showdown {
            self.status = PokerStatus::Showdown {
                player_id: self.next_on_game(0),
                first_card: true,
            };

            ActionRequest::Reveal {
                player_id: self.next_on_game(0),
                first_card: true,
            }
        } else {
            let missing_to_reveal = stage.cards_to_reveal() - 1;
            self.status = PokerStatus::Revealing {
                stage,
                missing_to_reveal,
            };
            ActionRequest::Deal(None)
        }
    }

    pub fn submit_revealed_cards(&mut self, _cards: ()) {
        // TODO: Implement cards find
        self.finish(vec![]);
    }

    pub fn submit_bet_action(
        &mut self,
        action: ActionResponse,
    ) -> Result<Option<ActionRequest>, PokerError> {
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
                            Ok(None)
                        } else {
                            Ok(Some(ActionRequest::BetFrom(next_player)))
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
                                todo!();
                            } else {
                                // Call
                                if action.player_id == until {
                                    // Finish betting round. All players call big blind bet.
                                    Ok(Some(self.start_stage(next_stage)))
                                } else {
                                    let next_player =
                                        self.next_on_game(self.next_player(action.player_id));
                                    if next_player != until || !raised {
                                        // Missing some players to place their bets.
                                        Ok(Some(ActionRequest::BetFrom(next_player)))
                                    } else {
                                        // Finish betting round. All players call last raised stake.
                                        Ok(Some(self.start_stage(next_stage)))
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
