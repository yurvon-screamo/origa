-- TrailBase user table schema for Origa
-- Execute this SQL in TrailBase SQL Editor (/_/admin/editor)

CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trailbase_id BLOB UNIQUE NOT NULL,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    native_language INTEGER NOT NULL DEFAULT 0,
    jlpt_progress TEXT CHECK(json_valid(jlpt_progress)),
    current_japanese_level INTEGER,
    duolingo_jwt_token TEXT,
    telegram_user_id INTEGER,
    reminders_enabled INTEGER NOT NULL DEFAULT 0,
    knowledge_set TEXT CHECK(json_valid(knowledge_set)) NOT NULL DEFAULT '{"study_cards":{},"lesson_history":[]}',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) STRICT;

-- Create index for faster lookups by trailbase_id
CREATE INDEX idx_user_trailbase_id ON user(trailbase_id);

-- Create index for faster lookups by email
CREATE INDEX idx_user_email ON user(email);

-- _ROW_.trailbase_id = _USER_.id
-- _REQ_.trailbase_id = _USER_.id
-- _ROW_.trailbase_id = _USER_.id AND _REQ_.trailbase_id = _USER_.id
-- _ROW_.trailbase_id = _USER_.id
