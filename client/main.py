import argparse
import json

from lib import App, register
from watcher import watch
from poker import Poker

CONTRACT = 'poker'

# TODO: Add help to commands


class PokerCli(App):
    @register(name="list")
    def list_all(self):
        rooms = [(int(room['id']), room['name'], room['status'])
                 for room in self.near.view("all_rooms", {})]

        if len(rooms) == 0:
            print("No rooms found.")
            return

        rooms.sort()
        for room_id, name, status in rooms:
            print(f"{room_id:>03} {name} {status}")

    @register
    def new_room(self, name):
        room_id = self.near.change("new_room", dict(name=name))
        print(f"Created room {name} with id {room_id}")

    @register
    def enter(self, room_id):
        room_id = int(room_id)
        result = self.near.change("enter", dict(room_id=room_id))
        print(result)
        watch(self.near, room_id)

    @register(name="start")
    def _start(self, room_id):
        room_id = int(room_id)
        result = self.near.change("start", dict(room_id=room_id))
        print(result)

    @register(short="t")
    def state(self, room_id):
        room_id = int(room_id)
        result = self.near.view("state", dict(room_id=room_id))
        print(result)

    @register
    def deck(self, room_id):
        room_id = int(room_id)
        result = self.near.view("deck_state", dict(room_id=room_id))
        print(result)

    @register
    def poker(self, room_id):
        room_id = int(room_id)
        result = self.near.view("poker_state", dict(room_id=room_id))
        print(result)

    @register(name="raise")
    def _raise(self, room_id, amount):
        room_id = int(room_id)
        amount = int(amount)
        self.near.change("submit_bet_action", dict(
            room_id=room_id, bet={"Stake": amount}))

    @register
    def fold(self, room_id):
        room_id = int(room_id)
        self.near.change("submit_bet_action", dict(
            room_id=room_id, bet="Fold"))


if __name__ == '__main__':
    parser = argparse.ArgumentParser('Poker game')
    parser.add_argument(
        'node_key', help="Path to validator key. Usually in neardev/*/<account_id>.json.")
    parser.add_argument('--contract', help="Contract to use", default=CONTRACT)

    args = parser.parse_args()

    app = PokerCli(args.node_key, args.contract)
    app.start()
