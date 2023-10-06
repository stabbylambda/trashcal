use itertools::Itertools;
use trashcal::{pickup::PickupType, pickup_calendar::PickupCalendar, trashcal_handler};

async fn make_live_request(input: &str) -> PickupCalendar {
    let request = lambda_http::request::from_str(input).expect("failed to create request");
    let response = trashcal_handler(request).await.expect("Failed to execute");
    let body = std::str::from_utf8(response.body()).expect("Should have a body");
    let body: PickupCalendar = serde_json::from_str(body).expect("Should be deserializable");
    body
}

#[tokio::test]
async fn orange_recycleonly() {
    let input = include_str!("./data/orange_recycleonly.json");
    let body = make_live_request(input).await;
    let map = body.pickups.iter().into_group_map_by(|x| x.name);
    // recycleonly isn't eligible for greens pickups
    assert_eq!(map[&PickupType::Trash].len(), 2);
    assert_eq!(map[&PickupType::Recyclables].len(), 1);
}

#[tokio::test]
async fn blue_oppweeks() {
    let input = include_str!("./data/blue_oppweeks.json");
    let body = make_live_request(input).await;

    let map = body.pickups.iter().into_group_map_by(|x| x.name);
    // oppweeks is eligible for both recycle and greens, so trash is both weeks
    assert_eq!(map[&PickupType::Trash].len(), 1);
    assert_eq!(map[&PickupType::Recyclables].len(), 1);
    assert_eq!(map[&PickupType::Organics].len(), 1);
}

#[tokio::test]
async fn blue_sameweeks() {
    let input = include_str!("./data/blue_sameweeks.json");
    let body = make_live_request(input).await;

    let map = body.pickups.iter().into_group_map_by(|x| x.name);
    // sameweeks is eligible for both recycle and greens, so trash is both weeks
    assert_eq!(map[&PickupType::Trash].len(), 1);
    assert_eq!(map[&PickupType::Recyclables].len(), 1);
    assert_eq!(map[&PickupType::Organics].len(), 1);
}

#[tokio::test]
async fn orange_oppweeks() {
    let input = include_str!("./data/orange_oppweeks.json");
    let body = make_live_request(input).await;

    let map = body.pickups.iter().into_group_map_by(|x| x.name);
    // oppweeks is eligible for both recycle and greens, so trash is both weeks
    assert_eq!(map[&PickupType::Trash].len(), 2);
    assert_eq!(map[&PickupType::Recyclables].len(), 1);
    assert_eq!(map[&PickupType::Organics].len(), 1);
}

#[tokio::test]
async fn orange_sameweeks() {
    let input = include_str!("./data/orange_sameweeks.json");
    let body = make_live_request(input).await;

    let map = body.pickups.iter().into_group_map_by(|x| x.name);
    // sameweeks is eligible for both recycle and greens, so trash is both weeks
    assert_eq!(map[&PickupType::Trash].len(), 2);
    assert_eq!(map[&PickupType::Recyclables].len(), 1);
    assert_eq!(map[&PickupType::Organics].len(), 1);
}
