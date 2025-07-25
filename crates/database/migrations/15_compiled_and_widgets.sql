CREATE TABLE addon_widget_content (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,
    id BLOB NOT NULL UNIQUE,

    addon_id INTEGER NOT NULL,

    data BLOB NOT NULL,
    version INTEGER NOT NULL,
    settings JSON NOT NULL DEFAULT '{}',

    title TEXT,
    description TEXT,
    thumbnail TEXT,

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,

    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE vissl_code_addon (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,

    addon_id INTEGER NOT NULL,
    widget_id INTEGER NOT NULL,

    visual_data JSON,
    script_data TEXT,

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,

    UNIQUE (addon_id, widget_id),
    FOREIGN KEY(widget_id) REFERENCES addon_widget_content(pk) ON DELETE CASCADE,
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE addon_widget_panel (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,
    id BLOB NOT NULL UNIQUE,

    addon_id INTEGER NOT NULL,
    addon_widget_id INTEGER NOT NULL,

    data BLOB NOT NULL,
    version INTEGER NOT NULL,

    title TEXT NOT NULL,
    settings JSON,

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,

    FOREIGN KEY(addon_widget_id) REFERENCES addon_widget_content(pk) ON DELETE CASCADE,
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE vissl_code_addon_panel (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,

    addon_id INTEGER NOT NULL,
    widget_id INTEGER,
    widget_panel_id INTEGER,

    visual_data JSON,
    script_data TEXT,

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,

    UNIQUE (addon_id, widget_panel_id),
    FOREIGN KEY(widget_id) REFERENCES addon_widget_content(pk) ON DELETE CASCADE,
    FOREIGN KEY(widget_panel_id) REFERENCES addon_widget_panel(pk) ON DELETE CASCADE,
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE addon_compiled (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,
    id BLOB NOT NULL UNIQUE,

    addon_id INTEGER NOT NULL,

    settings JSON NOT NULL DEFAULT '{}',

    type TEXT NOT NULL,
    version TEXT NOT NULL,

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    deleted_at DATETIME,

    UNIQUE (addon_id, version),
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE addon_compiled_widget (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,
    id BLOB NOT NULL UNIQUE,

    addon_id INTEGER NOT NULL,
    compiled_id INTEGER NOT NULL,
    widget_id INTEGER,

    data JSON NOT NULL,
    script TEXT,
    version INTEGER NOT NULL,
    settings JSON NOT NULL DEFAULT '{}',
    hash TEXT NOT NULL,

    title TEXT,
    description TEXT,
    thumbnail TEXT,

    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,

    FOREIGN KEY(compiled_id) REFERENCES addon_compiled(pk) ON DELETE CASCADE,
    FOREIGN KEY(widget_id) REFERENCES addon_widget_content(pk) ON DELETE CASCADE,
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE addon_compiled_page (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,
    id BLOB NOT NULL UNIQUE,

    addon_id INTEGER NOT NULL,
    compiled_id INTEGER NOT NULL,

    hash TEXT NOT NULL,

    data JSON NOT NULL,
    script JSON,
    settings JSON NOT NULL,
    version INTEGER NOT NULL,

    type_of INTEGER NOT NULL,
    path TEXT NOT NULL,
    display_name TEXT NOT NULL,

    FOREIGN KEY(compiled_id) REFERENCES addon_compiled(pk) ON DELETE CASCADE,
    FOREIGN KEY(addon_id) REFERENCES addon(id) ON DELETE CASCADE
);

CREATE TABLE generated_page_section_data (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,

    prompt TEXT,
    response TEXT,

    data JSON NOT NULL,

    generated_by INTEGER NOT NULL,
    compiled_prompt BLOB NOT NULL,

    created_at DATETIME NOT NULL
);

CREATE TABLE generated_page_data (
    pk INTEGER PRIMARY KEY AUTOINCREMENT,

    prompt TEXT,
    response TEXT,

    data JSON NOT NULL,

    generated_by INTEGER NOT NULL,
    compiled_prompt BLOB NOT NULL,

    created_at DATETIME NOT NULL
);