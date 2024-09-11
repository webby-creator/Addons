CREATE TABLE addon_permission
(
    addon_id INTEGER NOT NULL REFERENCES addon(id) ON DELETE CASCADE,

    scope TEXT,
    category TEXT,
    operation TEXT,
    info TEXT
);