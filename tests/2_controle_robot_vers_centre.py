TEAM_NAME = "vers centre"

def update(data):
    # Obligatoire de retourner un dictionnaire avec ces valeurs pour réaliser une action
    print(data["my_orientation"])
    return {
        "target_position": (50, 50), # position en coordonnées globales (par rapport au centre du terrain) de où on veut aller
        "power": 255, # puissance donnée aux moteurs
        "target_orientation": 180, # orientation à laquelle on souhaite aller
        "kick": False, # shoot dans la balle
    }
