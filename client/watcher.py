import logging
import threading
import time
import os
import hashlib

from cryptography import encrypt_and_shuffle, partial_decrypt, generate_secret_key
from poker import Poker
from utils import load, dump


class PokerRoomWatcher(threading.Thread):
    def __init__(self, near, room_id, ui):
        self.ui = ui
        self.room_id = room_id
        self.near = near
        self.poker = Poker(near, room_id)
        self.player_id = None
        self.cards = []
        self.load()
        super().__init__()

    def load(self):
        # Load cards
        self.cards = load(self.filename("cards")) or []
        self.ui.cards = self.cards

        # Load secret key
        self.secret_key = load(self.filename(
            "secret_key")) or generate_secret_key()
        self.secret_key = int(self.secret_key)
        dump(self.filename("secret_key"), self.secret_key)

    def find_player_id(self):
        players = self.poker.deck_state()['Ok']['players']
        if not self.near.account_id in players:
            logging.debug(
                f"{self.near.account_id} is not in game {self.room_id}. Found: {players}")
            return True
        else:
            self.player_id = players.index(
                self.near.account_id)
            logging.debug(
                f"{self.near.account_id} playing in game {self.room_id}. Found {players}")
            return False

    def update_state(self):
        self._state = self.poker.state()
        self._deck_state = self.poker.deck_state()
        self._poker_state = self.poker.poker_state()
        self.ui.update_state(self.room_id, self._state,
                             self._deck_state, self._poker_state)

    def is_deck_action(self):
        try:
            return self._state['Ok'] == 'DeckAction'
        except KeyError:
            return False

    def check_deck_shuffling(self):
        if not self.is_deck_action():
            return

        try:
            index = int(self._deck_state['Ok']['status']['Shuffling'])
        except KeyError:
            return

        if index != self.player_id:
            return

        partial_shuffle = self.poker.get_partial_shuffle()["Ok"]
        delta = 2 if self.player_id == 0 else 0
        partial_shuffle = [int(value) + delta for value in partial_shuffle]
        partial_shuffle = encrypt_and_shuffle(partial_shuffle, self.secret_key)
        partial_shuffle = [str(value) for value in partial_shuffle]
        self.poker.submit_partial_shuffle(partial_shuffle)

    def filename(self, mode):
        node_env = os.environ.get("NODE_ENV", "")
        chain_enc = f"{self.near.node_url}-{self.near.contract}-{node_env}"
        suffix = hashlib.md5(chain_enc.encode()).hexdigest()[:8]
        return f"{self.near.account_id}-{self.room_id}-{mode}-{suffix}"

    def on_receive_card(self, card):
        self.cards.append(card)
        dump(self.filename("cards"), self.cards)
        ui.update_card(self.room_id, card)

    def check_revealing(self):
        if not self.is_deck_action():
            return

        try:
            index = int(self._deck_state['Ok']
                        ['status']['Revealing']['turn'])
        except KeyError:
            return

        if index != self.player_id:
            return

        progress = int(self._deck_state['Ok']
                       ['status']['Revealing']['progress'])

        progress = str(partial_decrypt(progress, self.secret_key))

        if self._deck_state['Ok']['status']['Revealing']['receiver'] == self.player_id:
            self.on_receive_card(int(progress) - 2)
            self.poker.finish_reveal()
        else:
            self.poker.submit_reveal_part(progress)

    def step(self):
        if self.player_id is None:
            if not self.find_player_id():
                return

        self.update_state()
        self.check_deck_shuffling()
        self.check_revealing()

    def run(self):
        time_to_sleep = 1.

        while True:
            self.step()
            time.sleep(time_to_sleep)


WATCHING = set()


def watch(near, room_id, ui):
    if room_id in WATCHING:
        logging.debug(f"Already watching room: {room_id}")
        return

    WATCHING.add(room_id)
    PokerRoomWatcher(near, room_id, ui).start()
    logging.debug(f"Start watching room: {room_id}")
