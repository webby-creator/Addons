CREATE TABLE addon_widget_settings (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,

    website_id INTEGER NOT NULL,

    addon_id INTEGER NOT NULL,
    addon_widget_id INTEGER NOT NULL,

	-- Foreign key to the object id inside the website page
    object_id BLOB,

    -- The JSON blob containing only the settings that override the widget's defaults.
    settings JSON NOT NULL DEFAULT '{}',

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,

    FOREIGN KEY(addon_widget_id) REFERENCES addon_widget_content(pk) ON DELETE CASCADE,
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE,
    UNIQUE (pk, website_id)
);

CREATE INDEX idx_addon_widget_settings_website_id ON addon_widget_settings (website_id);
CREATE INDEX idx_addon_widget_settings_addon_widget_id ON addon_widget_settings (addon_widget_id);
