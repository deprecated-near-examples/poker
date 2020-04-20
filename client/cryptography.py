import random

MOD = 1000000007


def encrypt_and_shuffle(partial_shuffle, secret_key):
    partial_shuffle = [pow(num, secret_key, MOD) for num in partial_shuffle]
    random.shuffle(partial_shuffle)
    return partial_shuffle
