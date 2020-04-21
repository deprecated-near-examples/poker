import io
from lib import logging

SUITES = '♥♠♦♣'
VALUES = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A']


def card(num):
    num = int(num)
    return VALUES[num % 13] + SUITES[num // 13]


def get(dic, *keys):
    for key in keys:
        try:
            if not key in dic:
                return None
        except TypeError:
            return None
        dic = dic[key]
    return dic


def build_table(table_i):
    cols = [0] * len(table_i[0])
    for row in table_i:
        for ix, value in enumerate(row):
            cols[ix] = max(cols[ix], len(str(value)))

    for i in range(len(cols)):
        cols[i] += 2

    table = io.StringIO()

    first = True
    for row in table_i:
        if first:
            first = False
        else:
            first_row = True
            for col in cols:
                if first_row:
                    first_row = False
                else:
                    print("+", end="", file=table)
                print("-" * col, end="", file=table)
            print(file=table)

        first_row = True
        for ix, value in enumerate(row):
            if first_row:
                first_row = False
            else:
                print("|", end="", file=table)
            print(str(value).center(cols[ix]), end="", file=table)
        print(file=table)

    return table.getvalue()


def parse_id(value):
    if isinstance(value, str):
        return value
    elif isinstance(value, dict):
        return list(value.keys())[0]
    else:
        raise ValueError(f"Type not hadled {type(value)}({value})")


class PokerUI:
    def __init__(self):
        self.room_id = None
        self.account_id = None
        self.cards = []
        self.state = None
        self.deck_state = None
        self.poker_state = None
        self.turn = None
        self._last_status = None

    def set_account_id(self, account_id):
        self.account_id = account_id

    def enter(self, room_id):
        self.room_id = room_id

    def update_table(self):
        table = [
            ["Name", "Cards", "Total", "Staked", "Turn"],
        ]

    def update_state(self, room_id, state, deck_state, poker_state, turn):
        if room_id != self.room_id:
            return

        self.state = state
        self.deck_state = deck_state
        self.poker_state = poker_state
        self.turn = get(turn, 'Ok')
        logging.debug(repr(self.state))
        logging.debug(repr(self.deck_state))
        logging.debug(repr(self.poker_state))
        logging.debug(repr(self.turn))
        self.display(False)

    def update_card(self, room_id, card):
        if room_id != self.room_id:
            return

        self.cards.append(card)
        self.display()

    def get_action(self):
        g_state = get(self.state, 'Ok')

        if g_state == 'PokerAction':
            return parse_id(get(self.poker_state, 'Ok', 'status'))

        elif g_state == 'DeckAction':
            return parse_id(get(self.deck_state, 'Ok', 'status'))

        else:
            return g_state

    def get_revealed_cards(self):
        res = get(self.deck_state, 'Ok', 'revealed')

        if res is None:
            return []

        total_players = len(get(self.deck_state, 'Ok', 'players'))

        cards = []
        for card in res[2 * total_players:]:
            if card is not None:
                cards.append(card)
        return cards

    def display(self, force=True):
        status = io.StringIO()
        print(file=status)

        if self.state is not None:
            players = get(self.deck_state, 'Ok', 'players')
            tokens = get(self.poker_state, 'Ok', 'tokens')
            staked = get(self.poker_state, 'Ok', 'staked')
            folded = get(self.poker_state, 'Ok', 'folded')

            action = self.get_action()
            turn = self.turn

            if len(players) > 0:
                tables = [
                    ["Name", "Cards", "Total", "Staked", "On Game", "Turn"],
                ]

                if len(self.cards) > 0:
                    my_cards = ' '.join(map(card, self.cards))
                else:
                    my_cards = ''

                turn_by_player = False

                for ix, player in enumerate(players):
                    if player == self.account_id:
                        player += "(*)"
                        cards = my_cards
                    else:
                        cards = ""

                    row = [player, cards, tokens[ix],
                           staked[ix], not folded[ix], ""]

                    if turn == ix:
                        turn_by_player = True
                        row[5] = action

                    tables.append(row)

                print(build_table(tables), file=status)

                revealed_cards = self.get_revealed_cards()
                if len(revealed_cards) > 0:
                    print("Table:", ' '.join(
                        map(card, revealed_cards)), file=status)
                    print(file=status)

                if not turn_by_player:
                    print("Status:", action, file=status)
                    print(file=status)

        print(f"[{self.account_id}]>>> ", end="", file=status)

        cur_status = status.getvalue()
        if cur_status != self._last_status or force:
            print(cur_status, end="")
        self._last_status = cur_status


if __name__ == '__main__':
    print('♥')
