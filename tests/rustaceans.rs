use reqwest::{blocking::Client, StatusCode};
use serde_json::{json, Value};

pub mod common;

#[test]
fn test_get_rustaceans() {
    // * Setup
    let client = Client::new();
    let rustacean1: Value = common::create_test_rustacean(&client);
    let rustacean2: Value = common::create_test_rustacean(&client);
    // * Test
    let response = client
        .get(format!("{}/rustaceans", common::APP_HOST))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json: Value = response.json().unwrap();
    assert!(json.as_array().unwrap().contains(&rustacean1));
    assert!(json.as_array().unwrap().contains(&rustacean2));

    // * Cleanup
    common::delete_test_rustacean(&client, rustacean1);
    common::delete_test_rustacean(&client, rustacean2);
}

#[test]
fn test_create_rustaceans() {
    let client = Client::new();
    let response = client
        .post(format!("{}/rustaceans", common::APP_HOST))
        .json(&json!({
          "name":"Luke ho",
          "email": "luke@ho.com"
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let rustacean: Value = response.json().unwrap();
    assert_eq!(
        rustacean,
        json!({
          "id":rustacean["id"],
          "name":"Luke ho",
          "email": "luke@ho.com",
          "created_at": rustacean["created_at"],
        })
    );
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_view_rustaceans() {
    let client = Client::new();
    let rustacean: Value = common::create_test_rustacean(&client);

    let response = client
        .get(format!(
            "http://127.0.0.1:8000/rustaceans/{}",
            rustacean["id"]
        ))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let rustacean: Value = response.json().unwrap();
    assert_eq!(
        rustacean,
        json!({
          "id":rustacean["id"],
          "name":"Luke ho",
          "email": "luke@ho.com",
          "created_at": rustacean["created_at"],
        })
    );
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_update_rustaceans() {
    let client = Client::new();
    let rustacean: Value = common::create_test_rustacean(&client);

    let response = client
        .put(format!(
            "{}/rustaceans/{}",
            common::APP_HOST,
            rustacean["id"]
        ))
        .json(&json!({
          "name":"Luke hoooooo",
          "email": "luke@hoooooo.com"
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let rustacean: Value = response.json().unwrap();
    assert_eq!(
        rustacean,
        json!({
          "id":rustacean["id"],
          "name":"Luke hoooooo",
          "email": "luke@hoooooo.com",
          "created_at": rustacean["created_at"],
        })
    );
    common::delete_test_rustacean(&client, rustacean);
}

#[test]
fn test_delete_rustaceans() {
    let client = Client::new();
    let rustacean: Value = common::create_test_rustacean(&client);

    let response = client
        .delete(format!(
            "{}/rustaceans/{}",
            common::APP_HOST,
            rustacean["id"]
        ))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
