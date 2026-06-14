"""Core data types."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class Article:
    title: str
    link: str
    summary: str
    date_pub: str  # YYYY-MM-DD
    source: str
    fetched_at: str  # ISO 8601
    body: str | None = None
    category: str = "bird_flu"
