import logging
import threading
import time

from cryptography import encrypt_and_shuffle, partial_decrypt
from poker import Poker


class PokerRoomWatcher(threading.Thread):
    def __init__(self, near, room_id):
        self.room_id = room_id
        self.near = near
        self.poker = Poker(near, room_id)
        self.player_id = None
        super().__init__()

    # TODO: Persist secret keys using (account_id-contract_id-room_id-NODE_ENV.json)
    # Generate secret keys
    def load_secret_key(self):
        self.secret_key = 5

    def find_player_id(self):
        players = self.poker.deck_state()['Ok']['players']
        if not self.near.account_id in players:
            logging.debug(
                f"{self.near.account_id} is not in game {self.room_id}. Found: {players}")
            return True
        else:
            self.load_secret_key()
            self.player_id = players.index(
                self.near.account_id)
            logging.debug(
                f"{self.near.account_id} playing in game {self.room_id}. Found {players}")
            return False

    def update_state(self):
        self._state = self.poker.state()
        self._deck_state = self.poker.deck_state()
        self._poker_state = self.poker.poker_state()

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
            # TODO: Store card
            print("Card:", progress, int(progress) - 2)
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


def watch(near, room_id):
    if room_id in WATCHING:
        logging.debug(f"Already watching room: {room_id}")
        return

    WATCHING.add(room_id)
    PokerRoomWatcher(near, room_id).start()
    logging.debug(f"Start watching room: {room_id}")
