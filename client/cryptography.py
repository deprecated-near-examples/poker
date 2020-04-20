import random

MOD = 1000000007


def extended_gcd(a, b):
    s, old_s = 0, 1
    t, old_t = 1, 0
    r, old_r = b, a

    while r != 0:
        q = old_r // r
        old_r, r = r, old_r - q * r
        old_s, s = s, old_s - q * s
        old_t, t = t, old_t - q * t

    return old_r, old_s, old_t


def inverse(a, mod):
    g, x, y = extended_gcd(a, mod)
    assert g == 1
    return (x % mod + mod) % mod


def encrypt_and_shuffle(partial_shuffle, secret_key):
    partial_shuffle = [pow(num, secret_key, MOD) for num in partial_shuffle]
    random.shuffle(partial_shuffle)
    return partial_shuffle


def partial_decrypt(progress, secret_key):
    pw = inverse(secret_key, MOD - 1)
    return pow(progress, pw, MOD)
