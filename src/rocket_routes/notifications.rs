use crate::api::{created, ok, ApiError, ApiResult, CreatedApiResult};
use crate::auth::{record_access_scope, require_admin, AuthenticatedUser};
use crate::models::{NewNotification, Notification, NotificationView};
use crate::repositories::{InboxFilters, NotificationReceiptRepository, NotificationRepository};
use crate::rocket_routes::audit_logs::record_audit_log;
use crate::rocket_routes::DbConn;
use crate::validation::{validate_request, FieldViolation, Validate};
use chrono::{DateTime, Utc};
use rocket::response::status::NoContent;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket_db_pools::Connection;
use serde_json::json;
use utoipa::ToSchema;

#[derive(rocket::form::FromForm, Default)]
pub struct InboxQuery {
    pub state: Option<String>,
    pub search: Option<String>,
    pub source: Option<String>,
    pub severity: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

impl Validate for InboxQuery {
    fn validate(&self) -> Vec<FieldViolation> {
        let mut errors = Vec::new();
        for (field, value, choices) in [
            (
                "state",
                self.state.as_deref(),
                &["unread", "read", "snoozed", "dismissed", "all", "archived"][..],
            ),
            (
                "severity",
                self.severity.as_deref(),
                &["info", "warning", "critical"][..],
            ),
        ] {
            if value.is_some_and(|value| !choices.contains(&value)) {
                errors.push(FieldViolation::new(
                    field,
                    format!("must be one of {}", choices.join(", ")),
                ));
            }
        }
        crate::validation::max_optional_len(&mut errors, "search", &self.search, 200);
        crate::validation::max_optional_len(&mut errors, "source", &self.source, 64);
        if self
            .page
            .is_some_and(|value| !(1..=1_000_000).contains(&value))
        {
            errors.push(FieldViolation::new("page", "must be from 1 to 1000000"));
        }
        if self
            .page_size
            .is_some_and(|value| !(1..=100).contains(&value))
        {
            errors.push(FieldViolation::new("page_size", "must be from 1 to 100"));
        }
        errors
    }
}

#[derive(serde::Serialize, ToSchema)]
pub struct InboxResponse {
    pub items: Vec<NotificationView>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[rocket::get("/me/notifications?<query..>")]
pub async fn get_inbox(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    query: InboxQuery,
) -> ApiResult<InboxResponse> {
    let query = validate_request(query)?;
    let access = record_access_scope(&mut db, &auth).await?;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(25);
    let clean =
        |value: Option<String>| value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty());
    let filters = InboxFilters {
        state: query.state.unwrap_or_else(|| "unread".into()),
        search: clean(query.search),
        source: clean(query.source),
        severity: query.severity,
    };
    let (items, total) =
        NotificationRepository::inbox_for_access(&mut db, &access, &filters, page, page_size)
            .await?;
    ok(InboxResponse {
        items,
        total,
        page,
        page_size,
    })
}

#[derive(Deserialize, ToSchema)]
pub struct NotificationSnoozeRequest {
    pub snoozed_until: DateTime<Utc>,
}

impl Validate for NotificationSnoozeRequest {
    fn validate(&self) -> Vec<FieldViolation> {
        if self.snoozed_until <= Utc::now() {
            vec![FieldViolation::new(
                "snoozed_until",
                "must be in the future",
            )]
        } else {
            Vec::new()
        }
    }
}

#[rocket::get("/notifications")]
pub async fn get_notifications(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
) -> ApiResult<Vec<NotificationView>> {
    let access = record_access_scope(&mut db, &auth).await?;
    let notifications =
        NotificationRepository::find_actionable_for_access(&mut db, 100, None, None, &access)
            .await?;
    ok(notifications)
}

#[rocket::get("/notifications/<id>")]
pub async fn view_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
) -> ApiResult<NotificationView> {
    let notification = find_accessible_notification(&mut db, &auth, id).await?;
    let notification =
        NotificationRepository::view_for_user(&mut db, notification, auth.user.id).await?;
    ok(notification)
}

#[rocket::post("/notifications/<id>/read")]
pub async fn mark_notification_read(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
) -> ApiResult<NotificationView> {
    let notification = find_accessible_notification(&mut db, &auth, id).await?;
    let receipt = NotificationReceiptRepository::mark_read(&mut db, id, auth.user.id).await?;
    record_receipt_audit(&mut db, &auth, "mark_read", id, &receipt).await?;

    ok(NotificationView::from_record(notification, Some(receipt)))
}

#[rocket::post("/notifications/<id>/unread")]
pub async fn mark_notification_unread(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
) -> ApiResult<NotificationView> {
    let notification = find_accessible_notification(&mut db, &auth, id).await?;
    let receipt = NotificationReceiptRepository::mark_unread(&mut db, id, auth.user.id).await?;
    record_receipt_audit(&mut db, &auth, "mark_unread", id, &receipt).await?;

    ok(NotificationView::from_record(notification, Some(receipt)))
}

#[rocket::post("/notifications/<id>/dismiss")]
pub async fn dismiss_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
) -> ApiResult<NotificationView> {
    let notification = find_accessible_notification(&mut db, &auth, id).await?;
    let receipt = NotificationReceiptRepository::dismiss(&mut db, id, auth.user.id).await?;
    record_receipt_audit(&mut db, &auth, "dismiss", id, &receipt).await?;

    ok(NotificationView::from_record(notification, Some(receipt)))
}

#[rocket::post("/notifications/<id>/snooze", format = "json", data = "<request>")]
pub async fn snooze_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
    request: Json<NotificationSnoozeRequest>,
) -> ApiResult<NotificationView> {
    let notification = find_accessible_notification(&mut db, &auth, id).await?;
    let request = validate_request(request.into_inner())?;
    let receipt =
        NotificationReceiptRepository::snooze(&mut db, id, auth.user.id, request.snoozed_until)
            .await?;
    record_receipt_audit(&mut db, &auth, "snooze", id, &receipt).await?;

    ok(NotificationView::from_record(notification, Some(receipt)))
}

#[rocket::post("/notifications/<id>/restore")]
pub async fn restore_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
) -> ApiResult<NotificationView> {
    let notification = find_accessible_notification(&mut db, &auth, id).await?;
    let receipt = NotificationReceiptRepository::restore(&mut db, id, auth.user.id).await?;
    record_receipt_audit(&mut db, &auth, "restore", id, &receipt).await?;

    ok(NotificationView::from_record(notification, Some(receipt)))
}

#[rocket::post("/notifications", format = "json", data = "<new_notification>")]
pub async fn create_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    new_notification: Json<NewNotification>,
) -> CreatedApiResult<Notification> {
    require_admin(&auth)?;
    let new_notification = validate_request(new_notification.into_inner())?;
    let notification = NotificationRepository::create(&mut db, new_notification).await?;
    record_audit_log(
        &mut db,
        &auth,
        "create",
        "notification",
        notification.id,
        json!({
            "source": &notification.source,
            "severity": &notification.severity,
            "is_read": notification.is_read,
        }),
    )
    .await?;

    created(notification)
}

#[rocket::put("/notifications/<id>", format = "json", data = "<notification>")]
pub async fn update_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
    notification: Json<NewNotification>,
) -> ApiResult<Notification> {
    require_admin(&auth)?;
    let mut notification = validate_request(notification.into_inner())?;
    let existing = NotificationRepository::find(&mut db, id).await?;
    notification.connector_id = existing.connector_id;
    notification.owner_user_id = existing.owner_user_id;
    notification.maintainer_id = existing.maintainer_id;
    notification.source_updated_at = existing.source_updated_at;
    notification.last_seen_run_id = existing.last_seen_run_id;
    notification.archived_at = existing.archived_at;
    let notification = NotificationRepository::update(&mut db, id, notification).await?;
    record_audit_log(
        &mut db,
        &auth,
        "update",
        "notification",
        notification.id,
        json!({
            "source": &notification.source,
            "severity": &notification.severity,
            "is_read": notification.is_read,
        }),
    )
    .await?;

    ok(notification)
}

#[rocket::delete("/notifications/<id>")]
pub async fn delete_notification(
    auth: AuthenticatedUser,
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<NoContent, ApiError> {
    require_admin(&auth)?;
    let notification = NotificationRepository::find(&mut db, id).await?;
    NotificationRepository::delete(&mut db, id).await?;
    record_audit_log(
        &mut db,
        &auth,
        "delete",
        "notification",
        id,
        json!({
            "source": &notification.source,
            "severity": &notification.severity,
        }),
    )
    .await?;

    Ok(NoContent)
}

async fn find_accessible_notification(
    db: &mut Connection<DbConn>,
    auth: &AuthenticatedUser,
    id: i32,
) -> Result<Notification, ApiError> {
    let notification = NotificationRepository::find(db, id).await?;
    let access = record_access_scope(db, auth).await?;
    if !access.allows(notification.owner_user_id, notification.maintainer_id) {
        return Err(ApiError::NotFound);
    }

    Ok(notification)
}

async fn record_receipt_audit(
    db: &mut Connection<DbConn>,
    auth: &AuthenticatedUser,
    action: &str,
    notification_id: i32,
    receipt: &crate::models::NotificationReceipt,
) -> Result<(), ApiError> {
    record_audit_log(
        db,
        auth,
        action,
        "notification",
        notification_id,
        json!({
            "user_id": auth.user.id,
            "read_at": receipt.read_at,
            "dismissed_at": receipt.dismissed_at,
            "snoozed_until": receipt.snoozed_until,
        }),
    )
    .await
}
