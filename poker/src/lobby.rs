use crate::game::{Game, GameError, GameStatus};
use crate::types::{CryptoHash, RoomId};
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::near_bindgen;
use serde::Serialize;
use std::collections::HashMap;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct RoomInfo {
    name: String,
    id: RoomId,
    status: GameStatus,
}

impl From<&Game> for RoomInfo {
    fn from(poker: &Game) -> Self {
        Self {
            name: poker.name.clone(),
            id: poker.id,
            status: poker.status.clone(),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default, Serialize)]
pub struct Lobby {
    last_room: RoomId,
    rooms: HashMap<RoomId, Game>,
}

#[near_bindgen]
impl Lobby {
    pub fn new() -> Self {
        Self {
            last_room: 0,
            rooms: HashMap::new(),
        }
    }

    pub fn new_room(&mut self, name: String) -> RoomId {
        let room_id = self.last_room;
        self.last_room += 1;
        let poker = Game::new(name, room_id);
        self.rooms.insert(room_id, poker);
        room_id
    }

    pub fn all_rooms(&self) -> Vec<RoomInfo> {
        self.rooms.values().map(Into::into).collect()
    }

    pub fn all_active_rooms(&self) -> Vec<RoomInfo> {
        self.rooms
            .values()
            .filter_map(|val| {
                if val.status.is_active() {
                    Some(val)
                } else {
                    None
                }
            })
            .map(Into::into)
            .collect()
    }

    pub fn all_initiating_rooms(&self) -> Vec<RoomInfo> {
        self.rooms
            .values()
            .filter_map(|val| {
                if val.status.is_initiating() {
                    Some(val)
                } else {
                    None
                }
            })
            .map(Into::into)
            .collect()
    }

    fn room_ref(&self, room_id: RoomId) -> Result<&Game, GameError> {
        self.rooms.get(&room_id).ok_or(GameError::RoomIdNotFound)
    }

    fn room_mut(&mut self, room_id: RoomId) -> Result<&mut Game, GameError> {
        self.rooms
            .get_mut(&room_id)
            .ok_or(GameError::RoomIdNotFound)
    }
}

//Game interface for lobby
impl Lobby {
    pub fn enter(&mut self, room_id: RoomId) -> Result<(), GameError> {
        self.room_mut(room_id)?.enter()
    }

    pub fn start(&mut self, room_id: RoomId) -> Result<(), GameError> {
        self.room_mut(room_id)?.start()
    }

    pub fn close(&mut self, room_id: RoomId) -> Result<(), GameError> {
        self.room_mut(room_id)?.close()
    }
}

// Deck interface for lobby
impl Lobby {
    pub fn get_partial_shuffle(&self, room_id: RoomId) -> Result<Vec<CryptoHash>, GameError> {
        self.room_ref(room_id)?
            .get_partial_shuffle()
            .map_err(Into::into)
    }

    pub fn submit_shuffled(
        &mut self,
        room_id: RoomId,
        new_cards: Vec<CryptoHash>,
    ) -> Result<(), GameError> {
        self.room_mut(room_id)?
            .submit_shuffled(new_cards)
            .map_err(Into::into)
    }

    pub fn finish_reveal(&mut self, room_id: RoomId) -> Result<(), GameError> {
        self.room_mut(room_id)?.finish_reveal().map_err(Into::into)
    }

    pub fn submit_reveal_part(
        &mut self,
        room_id: RoomId,
        card: CryptoHash,
    ) -> Result<(), GameError> {
        self.room_mut(room_id)?
            .submit_reveal_part(card)
            .map_err(Into::into)
    }
}
