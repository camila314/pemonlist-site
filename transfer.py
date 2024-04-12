import urllib.request
import urllib.parse
import re
import json
import edgedb
import sys
from datetime import timedelta, datetime

client = edgedb.create_client()

def vid_id(link):
	url = urllib.parse.urlsplit(link)

	try:
		if re.match(r"(?:www\.)youtube\.com", url.netloc) != None:
			return urllib.parse.parse_qs(url.query)['v'][0]

		if re.match(r"youtu\.be", url.netloc) != None:
			return url.path[1::]
	except:
		print("    YouTube link not a video")

	return link

def transfer_level(path, placement):
	req = urllib.request.Request(
		url = path,
		headers = { 'User-Agent': 'Mozilla/5.0' }
	)

	data = json.load(urllib.request.urlopen(req))

	print(f"Transferring {level} : {data['id']}")

	verifier = client.query_single("select Player { id } filter .name = <str>$verifier", verifier=data["verifier"])
	if not verifier:
		verifier = client.query_single("""insert Player {
			name := <str>$verifier
		}""", verifier = data["verifier"])
	if not verifier:
		print("    nah what")
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
		name = data["name"],
		creator = data["author"],
		level_id = data["id"],
		video_id = video_id,
		placement = placement,
		verifier = verifier.id
	)

	for record in data["records"]:
		player = client.query_single("select Player { id } filter .name = <str>$player", player=record["user"])
		if not player:
			player = client.query_single("""insert Player {
				name := <str>$player,
				device := <Device>Device.Desktop
			}""", player = record["user"])

		dt = datetime.strptime(record["hz"], "%H:%M:%S.%f")
		delta = timedelta(hours=dt.hour, minutes=dt.minute, seconds=dt.second, microseconds=dt.microsecond)

		video_id = vid_id(record.get("link", ""))
		if (video_id == ""):
			print(f"    {record['user']}'s record video not set")
			continue

		client.query_single("""insert Entry {
			status := Status.Approved,
			video_id := <str>$video_id,
			player := <Player><uuid>$player,
			level := (select Level filter .level_id = <int64>$level_id),
			time := <duration>$time
		}""",
			player = player.id,
			level_id = data["id"],
			time = delta,
			video_id = video_id
		)

data_path = sys.argv[1]

req = urllib.request.Request(
	url = f"{data_path}/_list.json",
	headers = { 'User-Agent': 'Mozilla/5.0' }
)

rankings = json.load(urllib.request.urlopen(req))

client.execute("delete Entry")
client.execute("delete Level")
client.execute("delete Player")

for placement, level in enumerate(rankings):
	transfer_level(data_path + "/" + level + ".json", placement + 1)

print("Setting mobile players")

mobile = [
	'ZainAhmed',
	'Colouts',
	'ZynTagY_927',
	'SWB Pro',
	'Lexi'
]

for player in mobile:
	print(f"    {player}")
	client.execute("update Player filter .name = <str>$player set { device := Device.Mobile }", player = player)
	client.execute("update Entry filter .player.name = <str>$player set { mobile := true }", player = player)
