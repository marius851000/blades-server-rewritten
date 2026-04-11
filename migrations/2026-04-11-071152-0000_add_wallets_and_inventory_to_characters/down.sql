-- Remove inventory and wallet columns from the characters table
ALTER TABLE characters
    DROP COLUMN IF EXISTS inventory,
    DROP COLUMN IF EXISTS wallet;