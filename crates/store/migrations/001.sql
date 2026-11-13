BEGIN IMMEDIATE;
CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL,
    source TEXT NOT NULL,
    bundle_json TEXT NOT NULL,
    saved_ms INTEGER NOT NULL
);
CREATE TABLE events (
    session_id TEXT NOT NULL REFERENCES sessions(id),
    event_id TEXT NOT NULL,
    timestamp_ms INTEGER NOT NULL,
    event_json TEXT NOT NULL,
    PRIMARY KEY (session_id, event_id)
);
CREATE INDEX events_timeline ON events(session_id,timestamp_ms,event_id);
CREATE TABLE file_reports (
    id TEXT PRIMARY KEY NOT NULL,
    path TEXT NOT NULL,
    report_json TEXT NOT NULL,
    saved_ms INTEGER NOT NULL
);
PRAGMA user_version=1;
COMMIT;

