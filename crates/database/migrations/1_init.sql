CREATE TABLE addon
(
    id INTEGER NOT NULL,

    member_id INTEGER NOT NULL,

    guid TEXT NOT NULL,

    name TEXT NOT NULL,
    tag_line TEXT NOT NULL,
    description TEXT NOT NULL,

    icon INTEGER,
    -- REFERENCES addon_media(id) ON DELETE CASCADE,
    version TEXT NOT NULL,

    is_visible BOOLEAN NOT NULL,
    is_accepted BOOLEAN NOT NULL,

    install_count INTEGER NOT NULL,
    delete_reason TEXT,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    PRIMARY KEY ("id" AUTOINCREMENT)
);

CREATE TABLE media_upload
(
    id INTEGER NOT NULL,

    uploader_id INTEGER NOT NULL,

    file_name TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_type TEXT NOT NULL,

    media_width INTEGER,
    media_height INTEGER,
    media_duration INTEGER,

    has_thumbnail BOOLEAN NOT NULL,

    store_path TEXT NOT NULL,

    hash CHAR(64),

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    PRIMARY KEY ("id" AUTOINCREMENT)
);

CREATE TABLE addon_media
(
    id INTEGER NOT NULL,

    addon_id INTEGER NOT NULL REFERENCES addon(id) ON DELETE CASCADE,
    type_of INTEGER,

    upload_id INTEGER REFERENCES media_upload(id) ON DELETE CASCADE,
    embed_url TEXT,

    idx INTEGER NOT NULL DEFAULT -1,

    created_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    PRIMARY KEY ("id" AUTOINCREMENT)
);