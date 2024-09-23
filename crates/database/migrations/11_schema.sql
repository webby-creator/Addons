-- TODO: Add Unique Constraint
ALTER TABLE addon ADD COLUMN name_id TEXT;
UPDATE addon SET name_id = lower(name);

CREATE TABLE schema
(
    id INTEGER NOT NULL,

    name TEXT NOT NULL,

    addon_id INTEGER REFERENCES addon(id) ON DELETE CASCADE,

    primary_field TEXT NOT NULL,
    display_name TEXT NOT NULL,

    permissions JSON NOT NULL,

    version REAL NOT NULL,

    allowed_operations JSON NOT NULL,

    ttl INTEGER,
    default_sort JSON,
    views JSON NOT NULL DEFAULT '[]',

    store TEXT NOT NULL,

    fields JSON NOT NULL,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    UNIQUE(id, addon_id),
    PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE schema_data
(
    id INTEGER NOT NULL,

    addon_id INTEGER NOT NULL REFERENCES addon(id) ON DELETE CASCADE,
    schema_id INTEGER NOT NULL REFERENCES schema(id) ON DELETE CASCADE,

    public_id TEXT NOT NULL,

    field_text JSON,
    field_number JSON,
    field_url JSON,
    field_email JSON,
    field_address JSON,
    field_phone JSON,
    field_bool JSON,
    field_datetime JSON,
    field_date JSON,
    field_time JSON,
    field_rich_content JSON,
    field_rich_text JSON,
    field_reference JSON,
    field_multi_reference JSON,
    field_gallery JSON,
    field_document JSON,
    field_multi_document JSON,
    field_image JSON,
    field_video JSON,
    field_audio JSON,
    field_tags JSON,
    field_array JSON,
    field_object JSON,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,

    UNIQUE(addon_id, schema_id, public_id),
    PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE schema_data_tag
(
    id INTEGER NOT NULL,
    schema_id INTEGER REFERENCES schema(id) ON DELETE CASCADE,
    row_id TEXT NOT NULL,

    name TEXT NOT NULL,
    name_lower TEXT NOT NULL,
    color TEXT NOT NULL,

    UNIQUE(schema_id, row_id, name_lower),
    PRIMARY KEY("id" AUTOINCREMENT)
);