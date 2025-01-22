use reqwest::{blocking::Client, StatusCode};
use serde_json::{json, Value};

pub mod common;

#[test]
fn test_get_crates() {
    // * Setup
    let client = Client::new();
    let rustacean: Value = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);
    let b_crate = common::create_test_crate(&client, &rustacean);

    // * Test
    let response = client
        .get(format!("{}/crates", common::APP_HOST))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json: Value = response.json().unwrap();
    assert!(json.as_array().unwrap().contains(&a_crate));
    assert!(json.as_array().unwrap().contains(&b_crate));

    // * Cleanup
    common::delete_test_crate(&client, a_crate);
    common::delete_test_crate(&client, b_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_create_crate() {
    // * Setup
    let client = Client::new();
    let rustacean = common::create_test_rustacean(&client);

    // * Test
    let response = client
        .post(format!("{}/crates", common::APP_HOST))
        .json(&json!({
          "rustacean_id": rustacean["id"],
          "code": "luke",
          "name":"Luke ho",
          "version":"0.1",
          "description": "luke crate description",
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(
        a_crate,
        json!({
          "id":a_crate["id"],
          "code": "luke",
          "name":"Luke ho",
          "version":"0.1",
          "description": "luke crate description",
          "rustacean_id": rustacean["id"],
          "created_at": a_crate["created_at"],
        })
    );

    // * Cleanup
    common::delete_test_crate(&client, a_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_view_crate() {
    // * Setup
    let client = Client::new();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);

    // * Test
    let response = client
        .get(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(
        a_crate,
        json!({
          "id":a_crate["id"],
          "code": "luke",
          "name":"Luke ho",
          "version":"0.1",
          "description": "luke crate description",
          "rustacean_id": rustacean["id"],
          "created_at": a_crate["created_at"],
        })
    );

    // * Cleanup
    common::delete_test_crate(&client, a_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_update_crate() {
    // * Setup
    let client = Client::new();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);

    // * Test
    let response = client
        .put(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .json(&json!({
            "code": "lukeee",
            "name":"Lukeee ho",
            "version":"0.2",
            "description": "lukeee crate description",
            "rustacean_id": rustacean["id"],
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(
        a_crate,
        json!({
            "id":a_crate["id"],
            "code": "lukeee",
            "name":"Lukeee ho",
            "version":"0.2",
            "description": "lukeee crate description",
            "rustacean_id": rustacean["id"],
            "created_at": a_crate["created_at"],
        })
    );

    // * Test author-switching for a crate and a very long description.
    let rustacean2 = common::create_test_rustacean(&client);
    let response = client
        .put(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .json(&json!({
            "code": "lukeee",
            "name":"Lukeee ho",
            "version":"0.2",
            "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.", 
            "rustacean_id": rustacean2["id"],
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let a_crate: Value = response.json().unwrap();
    assert_eq!(
        a_crate,
        json!({
            "id":a_crate["id"],
            "code": "lukeee",
            "name":"Lukeee ho",
            "version":"0.2",
            "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
            "rustacean_id": rustacean2["id"],
            "created_at": a_crate["created_at"],
        })
    );

    // * Cleanup
    common::delete_test_crate(&client, a_crate);
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_delete_crate() {
    // * Setup
    let client = Client::new();
    let rustacean = common::create_test_rustacean(&client);
    let a_crate = common::create_test_crate(&client, &rustacean);

    // * Test
    let response = client
        .delete(format!("{}/crates/{}", common::APP_HOST, a_crate["id"]))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // * Cleanup
    common::delete_test_rustacean(&client, rustacean);
}
