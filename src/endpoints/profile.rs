/*use edgedb_tokio::Client;
use crate::db::Player;
use rocket::State;
use rocket::http::Status;
use rocket::response::content::RawJson;
use rocket::{Rocket, Build};

use crate::util::PrintErr;

#[get("/<name>")]
async fn get_profile(name: &str, client: &State<Client>) -> Result<RawJson<String>, Status> {
    let player: Player = Player::from_name(client, name).await
        .print_err()
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    serde_json::to_string(&player)
        .map(|s| RawJson(s))
        .print_err()
        .map_err(|_| Status::InternalServerError)
}

#[get("/<name>/entries")]
async fn get_profile_entries(name: &str, client: &State<Client>) -> Result<RawJson<String>, Status> {
    let player: Player = Player::from_name(client, name).await
        .print_err()
        .map_err(|_| Status::InternalServerError)?
        .ok_or(Status::NotFound)?;

    let entries = player.get_entries(client).await
        .print_err()
        .map_err(|_| Status::InternalServerError)?;

    // TODO: restrict records that arent verified

    serde_json::to_string(&entries)
        .map(|s| RawJson(s))
        .print_err()
        .map_err(|_| Status::InternalServerError)
}

#[get("/leaderboard")]
async fn get_leaderboard(client: &State<Client>) -> Result<RawJson<String>, Status> {
    let entries = Player::leaderboard(client).await
        .print_err()
        .map_err(|_| Status::InternalServerError)?;

    serde_json::to_string(&entries)
        .map(|s| RawJson(s))
        .print_err()
        .map_err(|_| Status::InternalServerError)
}

pub fn init(rock: Rocket<Build>) -> Rocket<Build> {
	rock.mount("/api/v1/profile/", routes![
        get_profile,
        get_profile_entries,
        get_leaderboard
    ])
}*/
