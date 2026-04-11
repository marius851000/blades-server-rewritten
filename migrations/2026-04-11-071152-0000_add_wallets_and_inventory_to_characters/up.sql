-- Add inventory and wallet columns to the characters table.
-- Both columns are of type JSONB and default to empty JSON objects.
-- This migration ensures that existing rows receive a valid default value.

ALTER TABLE characters
    ADD COLUMN inventory JSONB NOT NULL DEFAULT '{}',
    ADD COLUMN wallet JSONB NOT NULL DEFAULT '[]';
