CREATE TABLE addon_dashboard_page
(
    addon_id INTEGER NOT NULL REFERENCES addon(id) ON DELETE CASCADE,

    type TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,

    UNIQUE(addon_id, path)
);