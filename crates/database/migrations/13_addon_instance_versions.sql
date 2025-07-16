ALTER TABLE addon_instance ADD COLUMN version TEXT NOT NULL DEFAULT '';
ALTER TABLE addon_instance ADD COLUMN settings JSON;