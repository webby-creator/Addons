ALTER TABLE addon_instance RENAME TO addon_instance1;

CREATE TABLE addon_instance (
    id INTEGER NOT NULL,

    public_id TEXT NOT NULL UNIQUE,

    website_id INTEGER NOT NULL,
    website_uuid TEXT,

    is_setup BOOLEAN NOT NULL DEFAULT false,
    delete_reason TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,
    addon_id INTEGER REFERENCES addon(id) ON DELETE CASCADE,
    version TEXT NOT NULL DEFAULT '',
    settings JSON,
    PRIMARY KEY("id" AUTOINCREMENT),
    UNIQUE(website_id, addon_id)
);

INSERT INTO addon_instance SELECT * FROM addon_instance1;

DROP TABLE addon_instance1;

CREATE UNIQUE INDEX addon_inst_idx_test ON addon_instance (
    website_id, addon_id, ifnull(deleted_at, 0)
);