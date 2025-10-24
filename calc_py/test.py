from enum import Enum
import math
import random

class Hand(Enum):
    Rock = 0
    Paper = 1
    Scissors = 2

class Result(Enum):
    Lose = 0
    Draw = 1
    Win = 2

class Janken:
    hand_rate: list[list[list[float]]]
    last_hand: Hand
    second_last_hand: Hand
    history: list[Result]
    def __init__(self):
        self.reset()
    def reset(self):
        self.hand_rate = [[[1/3 for _ in Hand] for _ in Hand] for _ in Hand]
        self.last_hand = None
        self.second_last_hand = None
        self.history = []
    def predict(self) -> Hand:
        if self.second_last_hand is None:
            return Hand(random.randint(0, 2))
        rates = self.hand_rate[self.second_last_hand.value][self.last_hand.value]
        predicted_value = max(range(len(rates)), key=lambda i: rates[i])
        return Hand((predicted_value + 1) % 3)
    def adjust_rate(self):
        value = 1.5
        for result in range(min(len(self.history), 5)):
            if self.history[result] == Result.Win:
                value += 0.1
            elif self.history[result] == Result.Lose:
                value -= 0.1
        return value
    def play(self, hand: Hand) -> Result:
        predicted_hand = self.predict()
        if self.second_last_hand is not None:
            rate = self.hand_rate[self.second_last_hand.value][self.last_hand.value]
            rate[hand.value] *= self.adjust_rate()
            rate_sum = sum(rate)
            for i in range(len(rate)):
                rate[i] /= rate_sum
        self.second_last_hand = self.last_hand
        self.last_hand = hand
        if (hand.value - predicted_hand.value) % 3 == 1:
            result = Result.Win
        elif hand == predicted_hand:
            result = Result.Draw
        else:
            result = Result.Lose
        self.history.insert(0, result)
        return result

janken_game = Janken()
while True:
    player_input = input("Enter your hand (0: Rock, 1: Paper, 2: Scissors, q: Quit): ")
    if player_input == 'q':
        break
    try:
        player_hand = Hand(int(player_input))
    except (ValueError, KeyError):
        print("Invalid input. Please enter 0, 1, 2, or q.")
        continue
    result = janken_game.play(player_hand)
    if result == Result.Win:
        print("You win!")
    elif result == Result.Lose:
        print("You lose!")
    else:
        print("It's a draw!")

