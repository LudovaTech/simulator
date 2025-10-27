from random import randint
import rerun as rr # pip install rerun-sdk
import test2

TEAM_NAME = "hi there!"

print("Hello!")


i = 0

rr.init("simulator", recording_id="simulator", spawn=True)


def update(data):
    rr.log("logs", rr.TextLog("this entry has loglevel TRACE", level=rr.TextLogLevel.TRACE))

    return {
        "target_position": data["ball_position"],
             "power": randint(150, 255),
             "target_orientation": 90
    }

update({"ball_position": None})