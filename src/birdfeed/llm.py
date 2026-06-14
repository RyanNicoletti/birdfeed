"""Weekly newsletter summary via the Anthropic API."""

from __future__ import annotations

import datetime as dt
import logging

from . import config, db

log = logging.getLogger("birdfeed.llm")

_SYSTEM_PROMPT = (
    "You are a writer for a university newsletter that covers avian influenza "
    "(bird flu) and other emerging US viral and biodefense outbreaks (such as "
    "the New World screwworm, measles, mpox, Ebola, Marburg, Nipah, and similar "
    "threats) for an academic audience. Summarize the supplied articles into a "
    "single concise weekly update. Lead with the most significant developments "
    "(new human or mammal cases, major outbreaks, policy or vaccine news). Group "
    "related items, and keep avian influenza developments distinct from the other "
    "outbreaks. Write in clear, informative prose with short paragraphs. Do not "
    "use emojis, markdown headers, or bullet lists. If the articles conflict or "
    "are sparse, say so plainly rather than inventing detail."
)


class SummaryError(Exception):
    pass


def _format_articles(articles) -> str:
    chunks = []
    for a in articles:
        text = a.body or a.summary
        if not text:
            continue
        chunks.append(f"SOURCE: {a.source}\nTITLE: {a.title}\nLINK: {a.link}\n{text}")
    return "\n\n---\n\n".join(chunks)


def generate_summary() -> str:
    """Summarize the past week's articles and store the result. Returns the text."""
    if not config.ANTHROPIC_API_KEY:
        raise SummaryError("ANTHROPIC_API_KEY not set")

    articles = db.articles_since(config.SUMMARY_WINDOW_DAYS)
    corpus = _format_articles(articles)
    if not corpus.strip():
        raise SummaryError("no article content available to summarize")

    # Imported lazily so the web/scrape paths don't require the SDK at import time.
    import anthropic

    client = anthropic.Anthropic(api_key=config.ANTHROPIC_API_KEY)
    resp = client.messages.create(
        model=config.ANTHROPIC_MODEL,
        max_tokens=1500,
        system=[
            {
                "type": "text",
                "text": _SYSTEM_PROMPT,
                "cache_control": {"type": "ephemeral"},
            }
        ],
        messages=[
            {
                "role": "user",
                "content": (
                    "Here are this week's articles on avian influenza and other "
                    "emerging US viral and biodefense outbreaks. Write the weekly "
                    "update.\n\n" + corpus
                ),
            }
        ],
    )
    text = "".join(block.text for block in resp.content if block.type == "text").strip()
    if not text:
        raise SummaryError("model returned an empty summary")

    today = dt.date.today()
    start = today - dt.timedelta(days=config.SUMMARY_WINDOW_DAYS)
    date_range = f"{start.isoformat()} to {today.isoformat()}"
    db.insert_summary(text, date_range)
    log.info("stored weekly summary for %s", date_range)
    return text
