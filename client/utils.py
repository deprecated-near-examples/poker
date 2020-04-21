import json
import os

BASE = os.path.expanduser("~/.poker_near")


def dump(name, data):
    os.makedirs(BASE, exist_ok=True)
    with open(os.path.join(BASE, name + '.json'), 'w') as f:
        json.dump(data, f, indent=2)


def load(name):
    os.makedirs(BASE, exist_ok=True)
    target = os.path.join(BASE, name + '.json')

    if not os.path.exists(target):
        return None

    with open(target) as f:
        return json.load(f)
