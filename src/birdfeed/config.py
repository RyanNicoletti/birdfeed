"""Configuration and source definitions for birdfeed.

Values are read from the environment (optionally loaded from a .env file).
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from pathlib import Path

from dotenv import load_dotenv

load_dotenv()


def _db_path() -> str:
    """Resolve the SQLite database path.

    Prefers BIRDFEED_DB. Falls back to parsing a sqlite DATABASE_URL
    (for backwards compatibility with the old Rust config), else a local file.
    """
    explicit = os.getenv("BIRDFEED_DB")
    if explicit:
        return explicit
    url = os.getenv("DATABASE_URL", "")
    if url.startswith("sqlite:"):
        # sqlite:./x.db, sqlite:///abs/x.db -> strip scheme and leading slashes
        stripped = url[len("sqlite:") :]
        return stripped.lstrip("/") if not stripped.startswith("//") else "/" + stripped.lstrip("/")
    return "birdfeed.db"


DB_PATH: str = _db_path()
ANTHROPIC_API_KEY: str | None = os.getenv("ANTHROPIC_API_KEY")
ANTHROPIC_MODEL: str = os.getenv("ANTHROPIC_MODEL", "claude-sonnet-4-6")

HOST: str = os.getenv("BIRDFEED_HOST", "127.0.0.1")
PORT: int = int(os.getenv("BIRDFEED_PORT", "8080"))

# How many days of articles to show on the homepage.
DISPLAY_WINDOW_DAYS: int = int(os.getenv("BIRDFEED_DISPLAY_DAYS", "7"))
# How many days of articles to feed the weekly summary.
SUMMARY_WINDOW_DAYS: int = int(os.getenv("BIRDFEED_SUMMARY_DAYS", "7"))
# How many days of article records to retain; older rows are pruned each scrape.
RETENTION_WINDOW_DAYS: int = int(os.getenv("BIRDFEED_RETENTION_DAYS", "7"))

# Category identifiers persisted on each Article and used by the frontend.
CATEGORY_BIRD_FLU = "bird_flu"
CATEGORY_OTHER = "other"

# Title must contain one of these (case-insensitive) to count as an avian-flu story.
# Kept broad enough for mixed feeds (CDC/WHO/Google News) but flu-focused.
BIRD_FLU_KEYWORDS: tuple[str, ...] = (
    "avian influenza",
    "avian flu",
    "bird flu",
    "h5n1",
    "h5n2",
    "h5n5",
    "h5n9",
    "h7n",
    "hpai",
    "highly pathogenic",
    "h5 ",
)

# Title must contain one of these (case-insensitive) to count as another emerging
# US viral / biodefense outbreak story. Kept specific enough to avoid generic junk
# while covering the diseases and biosecurity terms the newsletter tracks.
OTHER_KEYWORDS: tuple[str, ...] = (
    "new world screwworm",
    "screwworm",
    "measles",
    "mpox",
    "monkeypox",
    "ebola",
    "marburg",
    "nipah",
    "lassa",
    "dengue",
    "west nile",
    "eastern equine encephalitis",
    "polio",
    "poliovirus",
    "cholera",
    "anthrax",
    "tularemia",
    "plague",
    "botulism",
    "hantavirus",
    "oropouche",
    "chikungunya",
    "zika",
    "select agent",
    "biosecurity",
    "biodefense",
)


@dataclass(frozen=True)
class Source:
    """A news source to scrape.

    kind:
      - "rss"        : a direct RSS/Atom feed with real article links.
      - "googlenews" : a Google News RSS query; links are resolved best-effort.
    pre_filtered: if True, every item is on-topic and the keyword filter is skipped
      (e.g. a feed that is already scoped to bird flu).
    """

    name: str
    url: str
    kind: str = "rss"
    pre_filtered: bool = False


# Reputable sources. Direct RSS feeds give real links + full article bodies; the
# Google News query carries the daily volume (mainstream wire + local press) that
# the low-frequency specialist feeds can't. Google News items are quality-gated
# downstream: roundup headlines are dropped at scrape time and re-titled
# near-duplicates are collapsed at insert time.
SOURCES: list[Source] = [
    Source(
        name="CIDRAP (Univ. of Minnesota)",
        url="https://www.cidrap.umn.edu/news/49/rss",
        pre_filtered=True,
    ),
    Source(
        name="Avian Flu Diary",
        url="https://afludiary.blogspot.com/feeds/posts/default",
        pre_filtered=False,
    ),
    Source(
        name="The Poultry Site",
        url="https://www.thepoultrysite.com/articles.rss",
        pre_filtered=False,
    ),
    Source(
        name="WHO News",
        url="https://www.who.int/rss-feeds/news-english.xml",
        pre_filtered=False,
    ),
    Source(
        # World Organisation for Animal Health — authoritative on HPAI as an
        # animal-disease event. General feed, so keyword-filtered.
        name="WOAH (World Org. for Animal Health)",
        url="https://www.woah.org/en/feed/",
        pre_filtered=False,
    ),
    Source(
        name="Google News",
        url=(
            "https://news.google.com/rss/search?"
            "q=%22avian+influenza%22+OR+%22bird+flu%22+OR+H5N1+when:2d"
            "&hl=en-US&gl=US&ceid=US:en"
        ),
        kind="googlenews",
        pre_filtered=False,
    ),
    Source(
        # Volume source for the other-outbreak category (screwworm, measles,
        # mpox, Ebola, Nipah, ...). Keyword-classified downstream like the
        # bird-flu Google News query.
        name="Google News (other outbreaks)",
        url=(
            "https://news.google.com/rss/search?"
            "q=%22New+World+screwworm%22+OR+screwworm+OR+measles+OR+mpox"
            "+OR+Ebola+OR+Marburg+OR+Nipah+OR+poliovirus+when:2d"
            "&hl=en-US&gl=US&ceid=US:en"
        ),
        kind="googlenews",
        pre_filtered=False,
    ),
]
