from random import randint
import rerun as rr # pip install rerun-sdk
# On peut importer les librairies qu'on veut et d'autres fichiers python dans le même répertoire

TEAM_NAME = "do logs"

# important de guarder les noms simulator sinon il ne se connecte pas
rr.init("simulator", recording_id="simulator", spawn=True)

def update(data):
    # cette fonctionnalité est très puissante et on peut en abuser pour tricher
    # partons du principe que cela n'arrivera pas car les codes seront relus à la main
    rr.log("logs", rr.TextLog("Example de log", level=rr.TextLogLevel.TRACE))

    return {
        "target_position": data["ball_position"],
        "power": randint(150, 255),
        "target_orientation": 90,
        "kick": False,
    }