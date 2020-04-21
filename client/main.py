import argparse
import json

from lib import App, register
from watcher import watch
from poker import Poker
from ui import PokerUI

CONTRACT = 'poker'


class PokerCli(App):
    @register(name="list", help="List all rooms.")
    def list_all(self):
        rooms = [(int(room['id']), room['name'], room['status'])
                 for room in self.near.view("all_rooms", {})]

        if len(rooms) == 0:
            print("No rooms found.")
            return

        rooms.sort()
        for room_id, name, status in rooms:
            print(f"{room_id:>03} {name} {status}")

    @register(help="<name> | Create a new room.")
    def new_room(self, name):
        room_id = self.near.change("new_room", dict(name=name))
        print(f"Created room {name} with id {room_id}")

    @register(help="<room_id> | Enter a room. Can only enter to play in rooms that are Initiating.")
    def enter(self, room_id):
        room_id = int(room_id)
        result = self.near.change("enter", dict(room_id=room_id))
        self.ui.enter(room_id)
        watch(self.near, room_id, self.ui)
        self.room_id = room_id

    @register(name="start", help="<room_id> | Start the game in a room if it is Initiating or Idle")
    def _start(self, room_id=None):
        if room_id is None:
            room_id = self.room_id
        room_id = int(room_id)
        result = self.near.change("start", dict(room_id=room_id))
        print(result)

    @register(name="raise", help="<amount> <room_id> | Increase your current bet TO amount.")
    def _raise(self, amount, room_id=None):
        if room_id is None:
            room_id = self.room_id
        room_id = int(room_id)
        amount = int(amount)
        result = self.near.change("submit_bet_action", dict(
            room_id=room_id, bet={"Stake": amount}))
        print(result)

    @register(help="<room_id> | Fold your cards for this round.")
    def fold(self, room_id=None):
        if room_id is None:
            room_id = self.room_id
        room_id = int(room_id)
        result = self.near.change("submit_bet_action", dict(
            room_id=room_id, bet="Fold"))
        print(result)

    @register(short="t", help="<room_id> | Show game state.")
    def state(self, room_id=None):
        if room_id is None:
            room_id = self.room_id
        room_id = int(room_id)
        result = self.near.view("state", dict(room_id=room_id))
        print(result)

    @register(help="<room_id> | Show raw deck state.")
    def deck_state(self, room_id=None):
        if room_id is None:
            room_id = self.room_id
        room_id = int(room_id)
        result = self.near.view("deck_state", dict(room_id=room_id))
        print(result)

    @register(help="<room_id> | Show raw poker table state.")
    def poker_state(self, room_id=None):
        if room_id is None:
            room_id = self.room_id
        room_id = int(room_id)
        result = self.near.view("poker_state", dict(room_id=room_id))
        print(result)


if __name__ == '__main__':
    parser = argparse.ArgumentParser('Poker game')
    parser.add_argument(
        'node_key', help="Path to validator key. Usually in neardev/*/<account_id>.json.")
    parser.add_argument('--contract', help="Contract to use", default=CONTRACT)
    parser.add_argument('--nodeUrl', help="NEAR Rpc endpoint")

    args = parser.parse_args()

    app = PokerCli(args.node_key, args.contract, args.nodeUrl, PokerUI())
    app.start()
