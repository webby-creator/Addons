ALTER TABLE addon_instance ADD COLUMN addon_id INTEGER REFERENCES addon(id) ON DELETE CASCADE;