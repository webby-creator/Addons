CREATE TABLE addon_instance
(
    id INTEGER NOT NULL,

    public_id TEXT NOT NULL UNIQUE,

    website_id INTEGER NOT NULL,
    website_uuid TEXT,

    is_setup BOOLEAN NOT NULL DEFAULT false,

    delete_reason TEXT,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    UNIQUE(website_id, deleted_at),
    PRIMARY KEY ("id" AUTOINCREMENT)
);

CREATE UNIQUE INDEX addon_inst_idx_test ON addon_instance (
    website_id, ifnull(deleted_at, 0)
);