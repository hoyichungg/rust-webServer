use reqwest::{blocking::Client, StatusCode};
use serde_json::{json, Value};

pub static APP_HOST: &'static str = "http://127.0.0.1:8000";

pub fn create_test_rustacean(client: &Client) -> Value {
    let response = client
        .post(format!("{}/rustaceans", APP_HOST))
        .json(&json!({
          "name":"Luke ho",
          "email": "luke@ho.com"
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    response.json().unwrap()
}

pub fn create_test_crate(client: &Client, rustacean: &Value) -> Value {
    let response = client
        .post(format!("{}/crates", APP_HOST))
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

    response.json().unwrap()
}

pub fn delete_test_rustacean(client: &Client, rustacean: Value) {
    let response = client
        .delete(format!("{}/rustaceans/{}", APP_HOST, rustacean["id"]))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

pub fn delete_test_crate(client: &Client, a_crate: Value) {
    let response = client
        .delete(format!("{}/crates/{}", APP_HOST, a_crate["id"]))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
