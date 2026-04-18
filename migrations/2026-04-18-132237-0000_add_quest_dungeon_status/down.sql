-- This file should undo anything in `up.sql`
ALTER TABLE quests
    DROP COLUMN IF EXISTS dungeon_state;
