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

	scalar type ProfileShape extending enum<Circle, Squircle, Square>;
	scalar type AccountStatus extending enum<None, Migrating, Done>;

	type Account extending Dated {
		multi link tokens := .<account[is AuthToken];

		required property status -> AccountStatus {
			default := AccountStatus.None;
		};

		required property image -> str {
			default := "";
		};
		required property profile_shape -> ProfileShape {
			default := ProfileShape.Circle
		};

		required property oauth2 -> str;
		required property email -> str;

		link player -> Player {
			default := <default::Player>{};
		};

		link discord -> Discord {
			default := <default::Discord>{};
		};

		required property mod -> bool {
			default := false;
		};
	}

	type AuthToken {
		required property token -> str;
		required property expires -> datetime {
			default := <datetime>(datetime_of_statement() + <cal::relative_duration>'7d');
		};

		required link account -> Account;
	}

	type Discord {
		required property user_id -> str;
		required property username -> str;
		required property global_name -> str;
		required property avatar -> str;
		required property accent_color -> str {
			default := '000000'
		};
		required property banner -> str {
			default := ""
		};
	}

	type MigrationRequest extending Dated {
		required link discord -> Discord;
		required link account -> Account;
		required link player -> Player;
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

	# function getPoints(place: int32) -> int32
	# 	 using (<int32>round(100.423 * 1000 ^ (1 / (place ^ (-1/3) + 2.178)) - 262.27*math::ln(10.82*place) + 0.639*place) if place < 200 else <int32>15);

	function getPoints(place: int32) -> int32
		using (<int32>math::ceil(32 * (place ^ (-1.33)) - 85.989 * (place ^ 0.2924) + 311.28 * (1.001495 ^ place)) if place < 200 else <int32>15);

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
