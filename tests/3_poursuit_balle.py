from random import randint

TEAM_NAME = "vers balle"

# data contient :
# my_position: (float, float)
# my_orientation: float, degrés entre -180 exclus et 180 inclus
# friend_position: (float, float)
# enemy1_position: (float, float)
# enemy2_position: (float, float)
# ball_position: (float, float)
def update(data):
    return {
        "target_position": data["ball_position"],
        "power": randint(150, 255), # pour que les robots ne soient pas bloqués au centre
        "target_orientation": 0,
        "kick": True,
    }