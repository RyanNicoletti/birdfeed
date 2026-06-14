"""SQLite persistence for articles and weekly summaries (stdlib sqlite3)."""

from __future__ import annotations

import datetime as dt
import sqlite3
from contextlib import contextmanager
from pathlib import Path

from . import config, dedup
from .models import Article

# How far back to look when checking whether an incoming title is a near-repeat
# of one we already stored. Re-titled duplicates always surface within a day or
# two, so a short window keeps the comparison cheap.
_DEDUP_WINDOW_DAYS = 21

_SCHEMA = """
CREATE TABLE IF NOT EXISTS articles (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL UNIQUE,
    link        TEXT NOT NULL UNIQUE,
    summary     TEXT NOT NULL DEFAULT '',
    body        TEXT,
    date_pub    TEXT NOT NULL,
    source      TEXT NOT NULL,
    fetched_at  TEXT NOT NULL,
    category    TEXT NOT NULL DEFAULT 'bird_flu'
);

CREATE INDEX IF NOT EXISTS idx_articles_date_pub ON articles (date_pub);

CREATE TABLE IF NOT EXISTS summaries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    summary     TEXT NOT NULL,
    date_range  TEXT NOT NULL UNIQUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
"""


@contextmanager
def _connect():
    Path(config.DB_PATH).parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(config.DB_PATH, timeout=30)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL;")
    try:
        yield conn
        conn.commit()
    finally:
        conn.close()


def init_db() -> None:
    with _connect() as conn:
        conn.executescript(_SCHEMA)
        # Idempotent migration for production DBs created before the category
        # column existed: add it (existing rows backfill to 'bird_flu' via the
        # DEFAULT) and ensure the supporting index exists.
        cols = {r["name"] for r in conn.execute("PRAGMA table_info(articles)")}
        if "category" not in cols:
            conn.execute(
                "ALTER TABLE articles ADD COLUMN category TEXT NOT NULL DEFAULT 'bird_flu'"
            )
        # Create the category index here (not in _SCHEMA) so it is built only
        # after the column is guaranteed to exist on migrated databases.
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_category_date "
            "ON articles (category, date_pub)"
        )


def insert_articles(articles: list[Article]) -> int:
    """Insert articles, skipping exact and near-duplicate titles/links.

    Exact title/link collisions are rejected by the table's UNIQUE constraints;
    re-titled near-duplicates (same story, lightly edited headline) are caught up
    front by comparing against recently stored titles and titles earlier in this
    batch. Returns the number of newly inserted rows.
    """
    if not articles:
        return 0
    inserted = 0
    with _connect() as conn:
        recent = conn.execute(
            "SELECT title FROM articles WHERE date_pub >= ?",
            (_cutoff(_DEDUP_WINDOW_DAYS),),
        ).fetchall()
        seen = [dedup.prepare(r["title"]) for r in recent]
        for a in articles:
            candidate = dedup.prepare(a.title)
            if dedup.is_near_duplicate(candidate, seen):
                continue
            cur = conn.execute(
                """
                INSERT OR IGNORE INTO articles
                    (title, link, summary, body, date_pub, source, fetched_at, category)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    a.title,
                    a.link,
                    a.summary,
                    a.body,
                    a.date_pub,
                    a.source,
                    a.fetched_at,
                    a.category,
                ),
            )
            if cur.rowcount:
                inserted += 1
                seen.append(candidate)
    return inserted


def _row_to_article(row: sqlite3.Row) -> Article:
    return Article(
        title=row["title"],
        link=row["link"],
        summary=row["summary"],
        body=row["body"],
        date_pub=row["date_pub"],
        source=row["source"],
        fetched_at=row["fetched_at"],
        category=row["category"],
        topic=config.topic_for_title(row["title"]),
    )


def _cutoff(days: int) -> str:
    return (dt.date.today() - dt.timedelta(days=days)).isoformat()


def articles_since(days: int, category: str | None = None) -> list[Article]:
    sql = (
        "SELECT title, link, summary, body, date_pub, source, fetched_at, category "
        "FROM articles WHERE date_pub >= ?"
    )
    params: list[str] = [_cutoff(days)]
    if category is not None:
        sql += " AND category = ?"
        params.append(category)
    sql += " ORDER BY date_pub DESC, source ASC"
    with _connect() as conn:
        rows = conn.execute(sql, params).fetchall()
    return [_row_to_article(r) for r in rows]


def articles_grouped_by_date(
    days: int, category: str | None = None
) -> list[tuple[str, list[Article]]]:
    """Return [(date, [articles])] for the homepage, newest date first.

    When `category` is given, only that category's articles are included.
    """
    grouped: dict[str, list[Article]] = {}
    for art in articles_since(days, category):
        day = art.date_pub[:10]
        grouped.setdefault(day, []).append(art)
    return sorted(grouped.items(), key=lambda kv: kv[0], reverse=True)


def prune_articles(days: int) -> int:
    """Delete article rows older than `days` and return the number removed.

    Only the articles table is pruned; summaries are never deleted.
    """
    with _connect() as conn:
        cur = conn.execute(
            "DELETE FROM articles WHERE date_pub < ?", (_cutoff(days),)
        )
        return cur.rowcount


def insert_summary(summary: str, date_range: str) -> None:
    with _connect() as conn:
        conn.execute(
            "INSERT OR IGNORE INTO summaries (summary, date_range) VALUES (?, ?)",
            (summary, date_range),
        )


def latest_summary() -> tuple[str, str] | None:
    """Return (summary, date_range) of the most recent weekly summary, or None."""
    with _connect() as conn:
        row = conn.execute(
            "SELECT summary, date_range FROM summaries ORDER BY created_at DESC LIMIT 1"
        ).fetchone()
    return (row["summary"], row["date_range"]) if row else None
