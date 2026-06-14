"""Per-source scraping: turn a feed into today's avian-flu Articles."""

from __future__ import annotations

import datetime as dt
import json
import logging
import re
import time

import feedparser
from bs4 import BeautifulSoup

from . import config, extract, http_client
from .models import Article

log = logging.getLogger("birdfeed.sources")


def _today() -> str:
    return dt.date.today().isoformat()


def _entry_date(entry) -> str | None:
    """Best-effort publish date as YYYY-MM-DD, or None if unknown."""
    for attr in ("published_parsed", "updated_parsed"):
        parsed = getattr(entry, attr, None)
        if parsed:
            return time.strftime("%Y-%m-%d", parsed)
    return None


def _classify(title: str) -> str | None:
    """Return the category for a title, or None if it matches neither keyword set.

    Bird-flu keywords win over the other set: an item that mentions both is
    treated as a bird-flu story.
    """
    t = title.lower()
    if any(kw in t for kw in config.BIRD_FLU_KEYWORDS):
        return config.CATEGORY_BIRD_FLU
    if any(kw in t for kw in config.OTHER_KEYWORDS):
        return config.CATEGORY_OTHER
    return None


# Digest/roundup headlines that bundle several unrelated stories and merely
# mention bird flu in passing. Either an explicit digest label, or several
# distinct headlines stitched together with semicolons.
_ROUNDUP_MARKERS = (
    "roundup",
    "round-up",
    "week in review",
    "weekly recap",
    "weekly vet report",
    "morning update",
    "morning medical update",
    "morning rounds",
    "news briefing",
    "daily briefing",
    "news in brief",
    "what we're reading",
)


def _is_roundup(title: str) -> bool:
    if title.count(";") >= 2:
        return True
    t = title.lower()
    return any(marker in t for marker in _ROUNDUP_MARKERS)


def _clean(text: str) -> str:
    return re.sub(r"\s+", " ", (text or "")).strip()


def _clean_summary(raw: str) -> str:
    """Strip any HTML from a feed description and return plain text.

    Feed descriptions often contain markup (and Google News stuffs a huge
    encoded URL into an <a href>), so we keep only the visible text.
    """
    if not raw:
        return ""
    return _clean(BeautifulSoup(raw, "html.parser").get_text(" "))


def scrape_source(source: config.Source) -> list[Article]:
    """Fetch one source and return today's matching articles.

    Per project policy the scraper only keeps items published *today* (the job
    runs twice daily, so a same-day filter avoids re-ingesting old items).
    """
    try:
        raw = http_client.get_bytes(source.url)
    except http_client.FetchError as exc:
        log.warning("fetch failed for %s: %s", source.name, exc)
        return []

    feed = feedparser.parse(raw)
    if feed.bozo and not feed.entries:
        log.warning("could not parse feed for %s: %s", source.name, feed.bozo_exception)
        return []

    today = _today()
    fetched_at = dt.datetime.now().astimezone().isoformat()
    articles: list[Article] = []

    for entry in feed.entries:
        title = _clean(getattr(entry, "title", ""))
        link = getattr(entry, "link", "")
        if not title or not link:
            continue

        date_pub = _entry_date(entry)
        if date_pub != today:
            continue

        category = _classify(title)
        if source.pre_filtered:
            # Pre-filtered feeds (e.g. CIDRAP) are already on-topic; keep every
            # item. Route clear OTHER matches accordingly and default the rest to
            # bird flu, since CIDRAP is a flu-leaning feed.
            if category is None:
                category = config.CATEGORY_BIRD_FLU
        elif category is None:
            # General feeds: keep only items matching one of the two keyword sets.
            continue

        if _is_roundup(title):
            log.debug("%s: skipping roundup headline: %s", source.name, title)
            continue

        if source.kind == "googlenews":
            link, src_name = _resolve_googlenews(entry, link)
            # Google News descriptions are just a redundant linked title; skip them.
            summary = ""
        else:
            src_name = source.name
            summary = _clean_summary(getattr(entry, "summary", ""))

        if not link.lower().startswith(("http://", "https://")):
            continue

        body = extract.fetch_body(link)

        articles.append(
            Article(
                title=title,
                link=link,
                summary=summary,
                date_pub=date_pub,
                source=src_name,
                fetched_at=fetched_at,
                body=body or None,
                category=category,
            )
        )

    log.info("%s: %d article(s) today", source.name, len(articles))
    return articles


def _resolve_googlenews(entry, gn_link: str) -> tuple[str, str]:
    """Resolve a Google News item to (real_url, publisher_name), best effort.

    Falls back to the Google News link (which still redirects in a browser) and
    a generic source label if decoding fails.
    """
    publisher = "Google News"
    src = getattr(entry, "source", None)
    if src is not None:
        title = src.get("title") if isinstance(src, dict) else getattr(src, "title", None)
        if title:
            publisher = f"{title} (via Google News)"

    real = _decode_google_news_url(gn_link)
    return (real or gn_link, publisher)


_GN_ID_RE = re.compile(r"/(?:articles|read)/([A-Za-z0-9_\-]+)")


def _decode_google_news_url(gn_url: str) -> str | None:
    """Decode a news.google.com/rss/articles/<id> link to the publisher URL.

    Uses Google News' internal batchexecute endpoint. Wrapped so any failure
    just returns None and the caller keeps the original link.
    """
    m = _GN_ID_RE.search(gn_url)
    if not m:
        return None
    article_id = m.group(1)
    try:
        html = http_client.get_text(f"https://news.google.com/articles/{article_id}")
        sig = re.search(r'data-n-a-sg="([^"]+)"', html)
        ts = re.search(r'data-n-a-ts="([^"]+)"', html)
        if not (sig and ts):
            return None
        payload = (
            "f.req="
            + json.dumps(
                [[["Fbv4je",
                   json.dumps(["garturlreq",
                               [["X", "X", ["X", "X"], None, None, 1, 1, "US:en",
                                 None, 1, None, None, None, None, None, 0, 1],
                                "X", "X", 1, [1, 1, 1], 1, 1, None, 0, 0, None, 0],
                               article_id, int(ts.group(1)), sig.group(1)]),
                   None, "generic"]]]
            )
        )
        from curl_cffi import requests as _cffi  # local import; optional dep

        resp = _cffi.post(
            "https://news.google.com/_/DotsSplashUi/data/batchexecute",
            data=payload,
            impersonate=http_client.IMPERSONATE,
            timeout=http_client.DEFAULT_TIMEOUT,
            headers={"content-type": "application/x-www-form-urlencoded;charset=UTF-8"},
        )
        for line in resp.text.splitlines():
            if "garturlres" in line:
                arr = json.loads(line)
                inner = json.loads(arr[0][2])
                return inner[1]
    except Exception as exc:  # noqa: BLE001 - decoding is best-effort
        log.debug("google news decode failed for %s: %s", gn_url, exc)
    return None
