ALTER TABLE users
    ADD COLUMN email_notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN push_notifications_enabled  BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN weekly_digest_enabled       BOOLEAN NOT NULL DEFAULT TRUE;
