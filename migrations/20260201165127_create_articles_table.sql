CREATE TABLE IF NOT EXISTS articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL UNIQUE,
    link TEXT NOT NULL UNIQUE,
    summary TEXT,
    date_pub TEXT,
    source TEXT,
    fetched_at TEXT
);
