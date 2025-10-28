TEAM_NAME = "vers centre"

def update(data):
    # Obligatoire de retourner un dictionnaire avec ces valeurs pour réaliser une action
    return {
        "target_position": (0, 0), # position en coordonnées globales (par rapport au centre du terrain) de où on veut aller
        "power": 255, # puissance donnée aux moteurs
        "target_orientation": 90 # orientation à laquelle on souhaite aller (non implémentée)
    }
