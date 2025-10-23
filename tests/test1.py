
TEAM_NAME = "hi there!"

print("Hello!")

import test2


def update(
        my_position,
        friend_position,
        enemy1_position,
        enemy2_position,
        ball_position,
        **_):
    return {
        "target_position": (0, 0),
        "power": 150,
        "target_orientation": 90
    }