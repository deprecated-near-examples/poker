import logging
import threading
import time

from cryptography import encrypt_and_shuffle


def view_function(function):
    def real_function(self):
        return self.near.view(function.__name__, dict(room_id=self.room_id))
    return real_function


class Poker:
    def __init__(self, near, room_id):
        self.near = near
        self.room_id = int(room_id)

    @view_function
    def state(self):
        pass

    @view_function
    def deck_state(self):
        pass

    @view_function
    def get_partial_shuffle(self):
        pass


class PokerRoomWatcher(threading.Thread):
    def __init__(self, near, room_id):
        self.room_id = room_id
        self.near = near
        self.poker = Poker(near, room_id)
        self.player_id = None
        super().__init__()

    # TODO: To read/write from file lock them first
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

    def check_deck_shuffling(self):
        try:
            index = int(self._state['Ok']['DeckAction']['Shuffling'])
        except:
            return

        if index != self.player_id:
            return

        partial_shuffle = self.poker.get_partial_shuffle()["Ok"]
        delta = 2 if self.player_id == 0 else 0
        partial_shuffle = [int(value) + delta for value in partial_shuffle]
        partial_shuffle = encrypt_and_shuffle(partial_shuffle, self.secret_key)
        partial_shuffle = [str(value) for value in partial_shuffle]

    def step(self):
        if self.player_id is None:
            if not self.find_player_id():
                return

        self._state = self.poker.state()
        self.check_deck_shuffling()

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
