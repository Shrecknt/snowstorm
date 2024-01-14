use crate::database::{player::PlayerInfo, server::PingResult};
use std::sync::mpsc::Sender;

pub async fn start_server(_sender: Sender<(PingResult, Vec<PlayerInfo>)>) {
    todo!() // listen with pnet for ping responses
}
