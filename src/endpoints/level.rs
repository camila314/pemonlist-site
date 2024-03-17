/*use edgedb_tokio::Client;
use crate::db::Level;
use rocket::State;
use rocket::http::Status;
use rocket::response::content::RawJson;
use rocket::{Rocket, Build};

use crate::util::PrintErr;

#[get("/list")]
async fn list_levels(client: &State<Client>) -> Result<RawJson<String>, Status> {
	let levels = Level::list(client).await
		.print_err()
		.map_err(|_| Status::InternalServerError)?;

	serde_json::to_string(&levels)
		.map(|s| RawJson(s))
		.print_err()
		.map_err(|_| Status::InternalServerError)
}

#[get("/<id>")]
async fn get_level(id: i32, client: &State<Client>) -> Result<RawJson<String>, Status> {
	let level: Level = Level::from_id(client, id).await
		.print_err()
		.map_err(|_| Status::InternalServerError)?
		.ok_or(Status::NotFound)?;

	serde_json::to_string(&level)
		.map(|s| RawJson(s))
		.print_err()
		.map_err(|_| Status::InternalServerError)
}

#[get("/<id>/entries")]
async fn get_level_entries(id: i32, client: &State<Client>) -> Result<RawJson<String>, Status> {
	let level: Level = Level::from_id(client, id).await
		.print_err()
		.map_err(|_| Status::InternalServerError)?
		.ok_or(Status::NotFound)?;

	let entries = level.get_entries(client).await
		.print_err()
		.map_err(|_| Status::InternalServerError)?;

	// TODO: restrict records that arent verified

	serde_json::to_string(&entries)
		.map(|s| RawJson(s))
		.print_err()
		.map_err(|_| Status::InternalServerError)
}

pub fn init(rock: Rocket<Build>) -> Rocket<Build> {
	rock.mount("/api/v1/level/", routes![
		get_level,
		list_levels,
		get_level_entries
	])
}*/
