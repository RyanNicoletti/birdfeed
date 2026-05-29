"""Orchestrate scraping across all configured sources."""

from __future__ import annotations

import logging

from . import config, db
from .models import Article
from .sources import scrape_source

log = logging.getLogger("birdfeed.scrape")


def run_scrape() -> int:
    """Scrape every source and persist new articles. Returns rows inserted."""
    db.init_db()
    collected: list[Article] = []
    for source in config.SOURCES:
        try:
            collected.extend(scrape_source(source))
        except Exception as exc:  # noqa: BLE001 - one bad source shouldn't abort the rest
            log.exception("source %s raised: %s", source.name, exc)

    inserted = db.insert_articles(collected)
    log.info("scrape complete: %d collected, %d new", len(collected), inserted)
    return inserted
