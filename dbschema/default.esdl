module default {
	abstract type Dated {
		required property created_at -> datetime {
			default := datetime_of_statement();
		}
	}

	scalar type Device extending enum<Mobile, Desktop, Both>;

	type Player extending Dated {
		required property name -> str {
			constraint exclusive;
		};

		multi link entries := .<player[is Entry];
		points := <int32>sum((select .entries filter .status = Status.Approved).level.points);
		rank := getPlayerRank(<Player>.id);
		required property device -> Device {
			default := Device.Both;
		};
	}

	scalar type Status extending enum<Submitted, Waiting, Investigating, Approved, Denied>;

	type Entry extending Dated {
		required property video_id -> str;
		required property time -> duration;

		required property status -> Status;
		required property mobile -> bool {
			default := false;
		};

		required link player -> Player;
		required link level -> Level;
		rank := getTimeRank(<Entry>.id)
	}

	scalar type ChangeType extending enum<Add, Edit, Remove>;
	type Changelog extending Dated {
		required property message -> str;
		required property change_type -> ChangeType;
	}

    function getPlayerRank(player: Player) -> int32
        using (<int32>(<int64>count(Player filter .points > player.points) + 1));

	function getTimeRank(entry: Entry) -> int32
        using (<int32>(<int64>count(Entry filter .time < entry.time and .level = entry.level) + 1));

	function getPoints(place: int32) -> int32
		using (<int32>round(100 * 1000 ^ (1 / (place ^ (-1/3) + 2.178)) - 262.27*math::ln(10.82*place) + 0.639*place));

	type Level extending Dated {
		required property name -> str;
		required property creator -> str;
		required property level_id -> int32 {
			constraint exclusive;
		};
		required property video_id -> str;
		required property placement -> int32 {
			constraint exclusive;
			default := (count(Level) + 1);
		};

		points := getPoints(.placement);

		required link verifier -> Player;

		multi link entries := .<level[is Entry];

		trigger level_add after insert for each do (
			insert Changelog {
				change_type := ChangeType.Add,
				message := (
					"Added level '" ++ __new__.name ++
					"' by " ++ __new__.creator ++
					" to placement " ++ <str>__new__.placement ++ "."
				)
			}
		);

		trigger level_remove after delete for each do (
			insert Changelog {
				change_type := ChangeType.Remove,
				message := (
					"Removed level '" ++ __old__.name ++
					"' by " ++ __old__.creator ++
					" from placement " ++ <str>__old__.placement ++ "."
				)
			}
		);

		trigger level_edit after update for each when (__old__.placement != __new__.placement) do (
			insert Changelog {
				change_type := ChangeType.Edit,
				message := (
					"Moved level '" ++ __new__.name ++
					"' by " ++ __new__.creator ++
					" from placement " ++ <str>__old__.placement ++
					" to " ++ <str>__new__.placement ++ "."
				)
			}
		);
	}
}
