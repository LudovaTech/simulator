from random import randint

TEAM_NAME = "hi there!"

print("Hello!")

import test2

i = 0

def update(data):
    # global i
    # i += 1
    # print(data)
    # if i < 1000:
    #     return {
    #         "target_position": (0, 0),
    #         "power": 100,
    #         "target_orientation": 90
    #     }
    # else:
    #     return {
    #         "target_position": (10, 0),
    #         "power": 100,
    #         "target_orientation": 90
    #     }
    return None
    return {
        "target_position": data["ball_position"],
             "power": randint(150, 255),
             "target_orientation": 90
    }