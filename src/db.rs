#![allow(dead_code)]

use edgedb_protocol::model::Uuid;
use edgedb_tokio::Client;
use edgedb_protocol::model::Datetime;
use edgedb_tokio::Queryable;
use serde::Serialize;

#[derive(Debug, PartialEq, Queryable, Serialize)]
pub struct Player {
	#[serde(skip_serializing)]
	id: Uuid,

	name: String,
	points: i32
}

#[derive(Debug, PartialEq, Queryable, Serialize)]
#[repr(u8)]
pub enum Status {
	Submitted = 0,
	Waiting = 1,
	Investigating = 2,
	Approved = 3,
	Denied = 4
}

#[derive(Debug, PartialEq, Queryable, Serialize)]
pub struct Entry {
	#[serde(skip_serializing)]
	created_at: Datetime,

	video: String,
	time: f64,
	status: Status,
	mobile: bool
}

#[derive(Debug, PartialEq, Queryable, Serialize)]
pub struct LeaderboardEntry {
	#[serde(skip_serializing)]
	created_at: Datetime,

	video: String,
	time: f64,
	status: Status,
	mobile: bool,
	level: Level
}

#[derive(Debug, PartialEq, Queryable, Serialize)]
pub struct LeaderboardPlayer {
	name: String,
	points: i32,
	entries: Vec<LeaderboardEntry>
}

impl Player {
	pub async fn from_id(client: &Client, id: i32) -> Result<Option<Player>, edgedb_tokio::Error> {
		client.query_single("select Player {
			id,
			name,
			points
		} filter .account_id = <int32>$0", &(id,)).await
	}

	pub async fn from_name(client: &Client, name: &str) -> Result<Option<Player>, edgedb_tokio::Error> {
		client.query_single("select Player {
			id,
			name,
			points
		} filter .name = <str>$0", &(name,)).await
	}

	pub async fn leaderboard(client: &Client) -> Result<Vec<LeaderboardPlayer>, edgedb_tokio::Error> {
		client.query("select Player {
			name,
			points,
			entries: {
				created_at,
				video,
				time,
				status,
				mobile,
				level: {
					id,
					created_at,
					name,
					creator,
					level_id,
					video,
					placement,
					points,
					verifier: {
						id,
						name,
						points
					}
				}
			}
		} filter .points > 0 order by .points desc", &()).await
	}

	pub async fn get_entries(&self, client: &Client) -> Result<Vec<Entry>, edgedb_tokio::Error> {
		client.query("select Entry {
			created_at,
			video,
			time,
			status,
			mobile
		} filter .player = <Player>$0", &(self.id,)).await
	}
}

#[derive(Debug, PartialEq, Queryable, Serialize)]
pub struct Level {
	#[serde(skip_serializing)]
	id: Uuid,
	#[serde(skip_serializing)]
	created_at: Datetime,

	name: String,
	creator: String,
	level_id: i32,
	video_id: String,
	placement: i32,
	points: i32,
	verifier: Player
}

impl Level {
	pub async fn from_id(client: &Client, id: i32) -> Result<Option<Level>, edgedb_tokio::Error> {
		client.query_single("select Level {
			id,
			created_at,
			name,
			creator,
			level_id,
			video_id,
			placement,
			points,
			verifier: {
				id,
				name,
				points
			}
		} filter .level_id = <int32>$0", &(id,)).await
	}

	pub async fn list(client: &Client) -> Result<Vec<Level>, edgedb_tokio::Error> {
		client.query("select Level {
			id,
			created_at,
			name,
			creator,
			level_id,
			video_id,
			placement,
			points,
			verifier: {
				id,
				name,
				points
			}
		}", &()).await
	}

	pub async fn get_entries(&self, client: &Client) -> Result<Vec<Entry>, edgedb_tokio::Error> {
		client.query("select Entry {
			created_at,
			video_id,
			time,
			status,
			mobile
		} filter .level = <Level>$0 order by .time", &(self.id,)).await
	}
}
