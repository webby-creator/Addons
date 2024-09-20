CREATE TABLE ref_widget
(
    addon_id INTEGER NOT NULL REFERENCES addon(id) ON DELETE CASCADE,

    widget_id INTEGER NOT NULL,
    public_id TEXT NOT NULL,

    UNIQUE(addon_id, widget_id)
);