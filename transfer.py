import os
import json
import edgedb
import sys
from datetime import timedelta, datetime

client = edgedb.create_client()

def vid_id(link):
	return link.replace("https://www.youtube.com/watch?v=", "").replace("https://youtu.be/", "")

def transfer_level(path, placement):
	data = json.load(open(path, "r"))

	print(f"Transferring {level} : {data['id']}")

	verifier = client.query_single("select Player { id } filter .name = <str>$verifier", verifier=data["verifier"])
	if not verifier:
		verifier = client.query_single("""insert Player {
			name := <str>$verifier
		}""", verifier = data["verifier"])
	if not verifier:
		print("nah what")
		return

	video_id = vid_id(data["verification"])

	client.query_single("""insert Level {
		name := <str>$name,
		creator := <str>$creator,
		level_id := <int64>$level_id,
		video_id := <str>$video_id,
		placement := <int64>$placement,
		verifier := <Player><uuid>$verifier
	}""",
		name=data["name"],
		creator=data["author"],
		level_id=data["id"],
		video_id=video_id,
		placement=placement,
		verifier=verifier.id
	)

	for record in data["records"]:
		player = client.query_single("select Player { id } filter .name = <str>$player", player=record["user"])
		if not player:
			player = client.query_single("""insert Player {
				name := <str>$player
			}""", player = record["user"])
		if not player:
			print("nah what")
			return

		dt = datetime.strptime(record["hz"], "%H:%M:%S.%f")
		delta = timedelta(hours=dt.hour, minutes=dt.minute, seconds=dt.second, microseconds=dt.microsecond)

		client.query_single("""insert Entry {
			status := Status.Approved,
			video_id := <str>$video_id,
			player := <Player><uuid>$player,
			level := (select Level filter .level_id = <int64>$level_id),
			time := <duration>$time
		}""",
			player=player.id,
			level_id=data["id"],
			time=delta,
			video_id=vid_id(record.get("link", ""))
		)

data_path = sys.argv[1]
rankings = json.load(open(data_path + "/_list.json", "r"))

client.execute("delete Entry")
client.execute("delete Level")
client.execute("delete Player")

for placement, level in enumerate(rankings):
	transfer_level(data_path + "/" + level + ".json", placement + 1)
