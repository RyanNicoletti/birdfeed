"""Extract readable article text from an HTML page."""

from __future__ import annotations

import logging

from bs4 import BeautifulSoup

from . import http_client

log = logging.getLogger("birdfeed.extract")

# Tags whose text is never article content.
_NOISE_TAGS = ("script", "style", "nav", "header", "footer", "aside", "form", "noscript")

# Preferred containers, in priority order. The first one found wins.
_CONTENT_SELECTORS = ("article", "main", '[role="main"]', ".article-body", ".entry-content")

MAX_BODY_CHARS = 12000


def _text_from(node) -> str:
    paragraphs = [p.get_text(" ", strip=True) for p in node.find_all("p")]
    paragraphs = [p for p in paragraphs if p]
    return "\n".join(paragraphs)


def extract_body(html: str) -> str:
    """Pull the main prose out of an HTML document.

    Prefers a semantic content container; falls back to all <p> tags.
    """
    soup = BeautifulSoup(html, "lxml")
    for tag in soup(_NOISE_TAGS):
        tag.decompose()

    best = ""
    for selector in _CONTENT_SELECTORS:
        node = soup.select_one(selector)
        if node:
            text = _text_from(node)
            if len(text) > len(best):
                best = text

    if len(best) < 200:
        # Weak/empty container: fall back to the whole document's paragraphs.
        whole = _text_from(soup)
        if len(whole) > len(best):
            best = whole

    return best[:MAX_BODY_CHARS].strip()


def fetch_body(url: str) -> str:
    """Fetch a URL and extract its article text. Returns '' on failure."""
    try:
        html = http_client.get_text(url)
    except http_client.FetchError as exc:
        log.debug("could not fetch body for %s: %s", url, exc)
        return ""
    try:
        return extract_body(html)
    except Exception as exc:  # noqa: BLE001 - bad HTML shouldn't kill the run
        log.debug("could not parse body for %s: %s", url, exc)
        return ""
