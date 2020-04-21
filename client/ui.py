class PokerUI:
    def __init__(self):
        self.room_id = None
        self.account_id = None
        self.cards = []
        self.state = None
        self.deck_state = None
        self.poker_state = None

    def set_account_id(self, account_id):
        self.account_id = account_id

    def enter(self, room_id):
        self.room_id = room_id

    def update_state(self, room_id, state, deck_state, poker_state):
        if room_id != self.room_id:
            return

        self.state = state
        self.deck_state = deck_state
        self.poker_state = poker_state
        # TODO: If is different state (call display)
        pass

    # TODO: Persist cards between games
    def update_card(self, room_id, card):
        if room_id != self.room_id:
            return
        self.cards.append(card)
        self.display()

    def display(self):
        print()
        print(self.cards)
        print(self.state)
        print(self.deck_state)
        print(self.poker_state)

        print("\n>>> ", end="")
