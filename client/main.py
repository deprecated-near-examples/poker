import argparse
import json

from lib import App, register

CONTRACT = 'poker'


class Deck(App):
    @register(name="list")
    def list_all(self):
        rooms = [(int(room['id']), room['name'], room['status'])
                 for room in self.near.view("all_rooms", {})]
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

    @register(name="start")
    def _start(self, room_id):
        room_id = int(room_id)
        result = self.near.change("start", dict(room_id=room_id))
        print(result)


if __name__ == '__main__':
    parser = argparse.ArgumentParser('Poker game')
    parser.add_argument(
        'node_key', help="Path to validator key. Usually in neardev/*/<account_id>.json.")
    parser.add_argument('--contract', help="Contract to use", default=CONTRACT)

    args = parser.parse_args()

    app = Deck(args.node_key, args.contract)
    app.start()
