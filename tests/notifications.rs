use chrono::{Duration, Utc};
use reqwest::{blocking::Client, StatusCode};
use serde_json::{json, Value};

pub mod common;
use common::CookieAuthRequest;

#[test]
fn inbox_filters_paginate_and_recover_personal_receipts() {
    let client = Client::new();
    let admin = common::create_admin_auth(&client);
    let member = common::create_test_auth(&client, "member");
    let other = common::create_test_auth(&client, "member");
    let source = common::unique_name("inbox");
    let mut records = Vec::new();
    for index in 0..6 {
        let response = client
            .post(format!("{}/notifications", common::APP_HOST))
            .cookie_auth(&admin.cookie)
            .json(&json!({
                "source": source, "title": format!("Inbox item {index}"),
                "body": if index == 0 { "Literal 100%_done\\path" } else { "Ordinary body" },
                "severity": if index == 0 { "critical" } else { "info" },
                "is_read": index == 5
            }))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        records.push(response.json::<Value>().unwrap()["data"].clone());
    }
    let id = |index: usize| records[index]["id"].as_i64().unwrap();
    // Set equal update times to exercise the id tie-breaker and simulate a
    // connector archive, which deliberately cannot be set through public CRUD.
    common::assert_safe_test_database();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
        let mut db = AsyncPgConnection::establish(common::database_url()).await.unwrap();
        diesel::sql_query("UPDATE notifications SET updated_at = '2026-09-06T00:00:00Z', archived_at = CASE WHEN id = $1 THEN now() ELSE NULL END WHERE source = $2")
            .bind::<diesel::sql_types::Integer, _>(id(4) as i32)
            .bind::<diesel::sql_types::Text, _>(&source)
            .execute(&mut db).await.unwrap();
    });
    let inbox = |cookie: &str, state: &str, page: &str, search: &str| {
        let response = client
            .get(format!("{}/me/notifications", common::APP_HOST))
            .cookie_auth(cookie)
            .query(&[
                ("source", source.as_str()),
                ("state", state),
                ("page", page),
                ("page_size", "2"),
                ("search", search),
            ])
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        response.json::<Value>().unwrap()["data"].clone()
    };
    let first = inbox(&member.cookie, "all", "1", "");
    let second = inbox(&member.cookie, "all", "2", "");
    let third = inbox(&member.cookie, "all", "3", "");
    assert_eq!(first["total"], 5);
    assert_eq!(first["page_size"], 2);
    let ids: Vec<i64> = [&first, &second, &third]
        .into_iter()
        .flat_map(|page| page["items"].as_array().unwrap())
        .map(|item| item["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![id(5), id(3), id(2), id(1), id(0)]);
    assert_eq!(inbox(&member.cookie, "all", "4", "")["items"], json!([]));
    assert_eq!(
        inbox(&member.cookie, "archived", "1", "")["items"][0]["id"],
        id(4)
    );
    assert_eq!(inbox(&member.cookie, "unread", "1", "")["total"], 4);
    assert_eq!(
        inbox(&member.cookie, "read", "1", "")["items"][0]["id"],
        id(5)
    );
    assert_eq!(
        inbox(&member.cookie, "all", "1", "%_DONE\\path")["total"],
        1
    );
    assert_eq!(inbox(&member.cookie, "all", "1", "missing")["total"], 0);
    post_action(&client, &member.cookie, id(0), "read");
    post_action(&client, &member.cookie, id(1), "dismiss");
    let response = client
        .post(format!(
            "{}/notifications/{}/snooze",
            common::APP_HOST,
            id(2)
        ))
        .cookie_auth(&member.cookie)
        .json(&json!({"snoozed_until": Utc::now() + Duration::hours(1)}))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(inbox(&member.cookie, "unread", "1", "")["total"], 1);
    assert_eq!(inbox(&other.cookie, "unread", "1", "")["total"], 4);
    assert_eq!(
        inbox(&member.cookie, "dismissed", "1", "")["items"][0]["id"],
        id(1)
    );
    assert_eq!(
        inbox(&member.cookie, "snoozed", "1", "")["items"][0]["id"],
        id(2)
    );
    assert_eq!(inbox(&other.cookie, "dismissed", "1", "")["total"], 0);
    post_action(&client, &member.cookie, id(1), "restore");
    post_action(&client, &member.cookie, id(2), "restore");
    assert_eq!(inbox(&member.cookie, "unread", "1", "")["total"], 3);
    for query in [
        "state=bogus",
        "page=0",
        "page=-1",
        "page_size=101",
        "page_size=0",
        "severity=bogus",
        "page=1000001",
    ] {
        let response = client
            .get(format!("{}/me/notifications?{query}", common::APP_HOST))
            .cookie_auth(&member.cookie)
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
        assert_eq!(
            response.json::<Value>().unwrap()["error"]["code"],
            "validation_failed"
        );
    }
    assert_eq!(
        client
            .get(format!("{}/me/notifications", common::APP_HOST))
            .send()
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    for record in records {
        common::delete_test_notification(&client, record);
    }
    for user in [member, other, admin] {
        common::delete_test_user(user.user_id);
    }
}

#[test]
fn test_notification_receipts_are_isolated_per_user() {
    let client = Client::new();
    let admin = common::create_admin_auth(&client);
    let first_user = common::create_test_auth(&client, "member");
    let second_user = common::create_test_auth(&client, "member");
    let source = common::unique_name("receipt_source");

    let response = client
        .post(format!("{}/notifications", common::APP_HOST))
        .cookie_auth(&admin.cookie)
        .json(&json!({
            "source": source,
            "external_id": common::unique_name("receipt_notification"),
            "title": "Review a private notification receipt",
            "body": "Each user should control only their own lifecycle state.",
            "severity": "warning",
            "is_read": false,
            "url": "https://erp.acme.test/messages/receipt-isolation"
        }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let notification: Value = response.json::<Value>().unwrap()["data"].clone();
    let notification_id = notification["id"].as_i64().unwrap();

    for token in [&first_user.cookie, &second_user.cookie] {
        let view = get_notification(&client, token, notification_id);
        assert_eq!(view["is_read"], false);
        assert_eq!(view["source_is_read"], false);
        assert_eq!(view["read_at"], Value::Null);
        assert_eq!(view["dismissed_at"], Value::Null);
        assert_eq!(view["snoozed_until"], Value::Null);
    }

    let response = post_action(&client, &first_user.cookie, notification_id, "read");
    assert_eq!(response["is_read"], true);
    assert_eq!(response["source_is_read"], false);
    assert!(response["read_at"].as_str().is_some());
    assert_actionable_state(&client, &first_user.cookie, &source, notification_id, false);
    assert_actionable_state(&client, &second_user.cookie, &source, notification_id, true);
    let second_user_view = get_notification(&client, &second_user.cookie, notification_id);
    assert_eq!(second_user_view["is_read"], false);
    assert_eq!(second_user_view["read_at"], Value::Null);

    let response = post_action(&client, &first_user.cookie, notification_id, "unread");
    assert_eq!(response["is_read"], false);
    assert_eq!(response["read_at"], Value::Null);
    assert_actionable_state(&client, &first_user.cookie, &source, notification_id, true);

    let snoozed_until = Utc::now() + Duration::hours(1);
    let response = client
        .post(format!(
            "{}/notifications/{notification_id}/snooze",
            common::APP_HOST
        ))
        .cookie_auth(&first_user.cookie)
        .json(&json!({ "snoozed_until": snoozed_until }))
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let snoozed: Value = response.json::<Value>().unwrap()["data"].clone();
    assert!(snoozed["snoozed_until"].as_str().is_some());
    assert_actionable_state(&client, &first_user.cookie, &source, notification_id, false);
    assert_actionable_state(&client, &second_user.cookie, &source, notification_id, true);

    let restored = post_action(&client, &first_user.cookie, notification_id, "restore");
    assert_eq!(restored["dismissed_at"], Value::Null);
    assert_eq!(restored["snoozed_until"], Value::Null);
    assert_actionable_state(&client, &first_user.cookie, &source, notification_id, true);

    let dismissed = post_action(&client, &first_user.cookie, notification_id, "dismiss");
    assert!(dismissed["dismissed_at"].as_str().is_some());
    assert_actionable_state(&client, &first_user.cookie, &source, notification_id, false);
    assert_actionable_state(&client, &second_user.cookie, &source, notification_id, true);
    assert_eq!(
        get_notification(&client, &second_user.cookie, notification_id)["dismissed_at"],
        Value::Null
    );

    let response = client
        .get(format!(
            "{}/audit-logs?action=dismiss&resource_type=notification&resource_id={notification_id}",
            common::APP_HOST
        ))
        .cookie_auth(&admin.cookie)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let audit_logs: Value = response.json::<Value>().unwrap()["data"].clone();
    assert!(audit_logs.as_array().unwrap().iter().any(|entry| {
        entry["actor_user_id"].as_i64() == Some(first_user.user_id as i64)
            && entry["resource_id"].as_str() == Some(notification_id.to_string().as_str())
    }));

    common::delete_test_notification(&client, notification);
    common::delete_test_user(second_user.user_id);
    common::delete_test_user(first_user.user_id);
    common::delete_test_user(admin.user_id);
}

fn post_action(client: &Client, token: &str, notification_id: i64, action: &str) -> Value {
    let response = client
        .post(format!(
            "{}/notifications/{notification_id}/{action}",
            common::APP_HOST
        ))
        .cookie_auth(token)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    response.json::<Value>().unwrap()["data"].clone()
}

fn get_notification(client: &Client, token: &str, notification_id: i64) -> Value {
    let response = client
        .get(format!(
            "{}/notifications/{notification_id}",
            common::APP_HOST
        ))
        .cookie_auth(token)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    response.json::<Value>().unwrap()["data"].clone()
}

fn assert_actionable_state(
    client: &Client,
    token: &str,
    source: &str,
    notification_id: i64,
    expected: bool,
) {
    let response = client
        .get(format!("{}/notifications", common::APP_HOST))
        .cookie_auth(token)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        contains_id(&response.json::<Value>().unwrap()["data"], notification_id),
        expected
    );

    let response = client
        .get(format!("{}/dashboard?source={source}", common::APP_HOST))
        .cookie_auth(token)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let dashboard: Value = response.json::<Value>().unwrap()["data"].clone();
    assert_eq!(
        contains_id(&dashboard["notifications"], notification_id),
        expected
    );
    assert_eq!(
        dashboard["summary"]["unread_notifications"].as_i64(),
        Some(i64::from(expected))
    );

    let response = client
        .get(format!("{}/me/overview", common::APP_HOST))
        .cookie_auth(token)
        .send()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let overview: Value = response.json::<Value>().unwrap()["data"].clone();
    assert_eq!(
        contains_id(&overview["unread_notifications"], notification_id),
        expected
    );
}

fn contains_id(items: &Value, id: i64) -> bool {
    items
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"].as_i64() == Some(id))
}
