use mowojang::{check_username, check_uuid, valid_geyser_username, valid_java_username};
use uuid::uuid;

#[tokio::main]
async fn main() {
    let username = "Shrecknt";
    let uuid = uuid!("281be5664aca46a6950181f8364fab56");
    let expected = mowojang::MowojangApiResponse::new(username, uuid);

    let is_java_username = valid_java_username(username);
    dbg!(is_java_username);
    let is_bedrock_username = valid_geyser_username(username);
    dbg!(is_bedrock_username);

    assert!(is_java_username);
    assert!(!is_bedrock_username);

    let from_username = check_username(username).await;
    dbg!(&from_username);
    let from_uuid = check_uuid(uuid).await;
    dbg!(&from_uuid);

    assert_eq!(from_username, Some(expected.clone()));
    assert_eq!(from_uuid, Some(expected.clone()));

    println!("Everything seems to work!");
}
