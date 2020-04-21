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
    def poker_state(self):
        pass

    @view_function
    def get_partial_shuffle(self):
        pass

    @view_function
    def get_turn(self):
        pass

    def submit_partial_shuffle(self, partial_shuffle):
        self.near.change("submit_shuffled", dict(
            room_id=self.room_id, new_cards=partial_shuffle))

    def submit_reveal_part(self, progress):
        self.near.change("submit_reveal_part", dict(
            room_id=self.room_id, card=progress))

    def finish_reveal(self):
        self.near.change("finish_reveal", dict(room_id=self.room_id))
