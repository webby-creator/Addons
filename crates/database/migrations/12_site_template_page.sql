
CREATE TABLE template_page
(
    id INTEGER NOT NULL,

    addon_id INTEGER NOT NULL REFERENCES addon(id) ON DELETE CASCADE,

    public_id TEXT NOT NULL,

    path TEXT NOT NULL,
    display_name TEXT NOT NULL,

    object_ids JSON NOT NULL,
    settings JSON NOT NULL,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,

    PRIMARY KEY ("id" AUTOINCREMENT)
);

CREATE TABLE template_page_content
(
    template_page_id INTEGER NOT NULL REFERENCES template_page(id) ON DELETE CASCADE UNIQUE,

    content TEXT NOT NULL,
    version INTEGER NOT NULL,

    updated_at TIMESTAMP NOT NULL
);