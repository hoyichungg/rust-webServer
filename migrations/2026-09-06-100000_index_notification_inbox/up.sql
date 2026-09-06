CREATE INDEX notifications_inbox_updated_id_idx
    ON notifications (updated_at DESC, id DESC) WHERE archived_at IS NULL;
CREATE INDEX notifications_inbox_source_updated_id_idx
    ON notifications (source, updated_at DESC, id DESC) WHERE archived_at IS NULL;
