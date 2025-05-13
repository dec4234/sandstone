#[allow(unused_imports)]

use crate::util::mojang::{get_player_details, get_uuid_from_username, get_uuids_from_usernames};

/*
These tests are ignored due to rate limit issues
 */

#[ignore]
#[tokio::test]
pub async fn test_get_uuid_from_name() {
	let response = get_uuid_from_username("Notch".to_string()).await.unwrap();
	
	assert_eq!(response.id, "069a79f444e94726a5befca90e38aaf5");
	assert_eq!(response.name, "Notch");
	assert_eq!(response.legacy, None);
	assert_eq!(response.demo, None);
	
	let response = get_uuid_from_username("jeb_".to_string()).await.unwrap();
	
	assert_eq!(response.id, "853c80ef3c3749fdaa49938b674adae6");
	assert_eq!(response.name, "jeb_");
	assert_eq!(response.legacy, None);
	assert_eq!(response.demo, None);
	
	tokio::time::sleep(std::time::Duration::from_secs(1)).await; // too fast
	
	let response = get_uuid_from_username("dec4234".to_string()).await.unwrap();
	
	assert_eq!(response.id, "ef39c1973c3d4776a22622096378a966");
	assert_eq!(response.name, "dec4234");
	assert_eq!(response.legacy, None);
	assert_eq!(response.demo, None);
	
	let response = get_uuid_from_username("dinnerbone".to_string()).await.unwrap();
	
	assert_eq!(response.id, "61699b2ed3274a019f1e0ea8c3f06bc6");
	assert_eq!(response.name, "Dinnerbone");
	assert_eq!(response.legacy, None);
	assert_eq!(response.demo, None);
	
	tokio::time::sleep(std::time::Duration::from_secs(1)).await; // too fast
	
	// people that don't exist -- should return an error that the given username doesn't exist
	get_uuid_from_username("37g43g3i4yid".to_string()).await.expect_err("Expected error");
	get_uuid_from_username("sdewve3veyirv3r".to_string()).await.expect_err("Expected error");
	get_uuid_from_username("22___343n3irt72".to_string()).await.expect_err("Expected error");
	get_uuid_from_username("ABC__23n2al1l".to_string()).await.expect_err("Expected error");
}

#[ignore]
#[tokio::test]
pub async fn test_bulk_usernames() {
	let resp = get_uuids_from_usernames(vec!["Notch".to_string(), "jeb_".to_string(), "dec4234".to_string(), "dinnerbone".to_string()]).await.unwrap();
	
	assert_eq!(resp.len(), 4);
	assert_eq!(resp[0].id, "ef39c1973c3d4776a22622096378a966");
	assert_eq!(resp[0].name, "dec4234");
	
	assert_eq!(resp[1].id, "61699b2ed3274a019f1e0ea8c3f06bc6");
	assert_eq!(resp[1].name, "Dinnerbone");
	
	assert_eq!(resp[2].id, "853c80ef3c3749fdaa49938b674adae6");
	assert_eq!(resp[2].name, "jeb_");
	
	assert_eq!(resp[3].id, "069a79f444e94726a5befca90e38aaf5");
	assert_eq!(resp[3].name, "Notch");
}

#[ignore]
#[tokio::test]
pub async fn test_get_player_details() {
	let response = get_player_details("ef39c1973c3d4776a22622096378a966".to_string()).await.unwrap();
	assert_eq!(response.id, "ef39c1973c3d4776a22622096378a966");
	assert_eq!(response.name, "dec4234");
	assert_eq!(response.legacy, None);
	assert_eq!(response.profileActions.len(), 0);
	
	assert_eq!(response.properties.len(), 1);
	assert_eq!(response.properties[0].name, "textures");
	
	let _texture = response.properties[0].get_skin_details().unwrap();
}