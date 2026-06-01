"""Near-duplicate detection for article titles.

Open aggregators (Google News) re-surface the same story with lightly edited
headlines, each carrying a distinct redirect URL — so they slip past the exact
title/link uniqueness in the DB. We collapse these by comparing *canonicalised*
headlines.

The tricky case is templated headlines that differ only by a place or number
("...detected in Iowa" vs "...in Ohio") — those are *distinct* events and must
NOT be merged, even though they're ~98% identical as strings. So a high string
similarity alone isn't enough: we additionally require that every differing word
is a trivial lowercase token. If the difference includes a proper noun or a
number, we treat the items as different stories and keep both.
"""

from __future__ import annotations

import re
import string
from difflib import SequenceMatcher

# Minimum string similarity (on the canonical headline) to even consider two
# titles the same story. Below this they're clearly different.
_SIMILARITY_FLOOR = 0.88

# Trailing publisher attribution that Google News (and some feeds) append.
_PUBLISHER_SEPS = (" - ", " – ", " — ", " | ")
_PUNCT_RE = re.compile(r"[^\w\s]")
_WS_RE = re.compile(r"\s+")


def _strip_publisher(title: str) -> str:
    """Drop a trailing ' - Publisher' style attribution, if present.

    Splits on the *rightmost* separator so headlines that themselves contain a
    dash keep everything up to the final publisher segment.
    """
    cut = max((title.rfind(sep) for sep in _PUBLISHER_SEPS), default=-1)
    return title[:cut] if cut != -1 else title


def _canonical(headline: str) -> str:
    """Lowercase, strip punctuation, collapse whitespace."""
    return _WS_RE.sub(" ", _PUNCT_RE.sub(" ", headline.lower())).strip()


def _diff_is_significant(head_a: str, head_b: str) -> bool:
    """True if the words that differ between two headlines look meaningful.

    A meaningful difference is a proper noun (capitalised word) or a number,
    which signals a distinct event rather than a re-titled duplicate.
    """
    diff = set(head_a.split()) ^ set(head_b.split())
    for token in diff:
        word = token.strip(string.punctuation)
        if not word:
            continue
        if any(ch.isdigit() for ch in word):
            return True
        if word[0].isupper():
            return True
    return False


def prepare(title: str) -> tuple[str, str]:
    """Pre-compute the (canonical, publisher-stripped headline) pair for a title.

    Callers normalise their existing corpus once and reuse the result.
    """
    headline = _strip_publisher(title)
    return _canonical(headline), headline


def is_near_duplicate(
    candidate: tuple[str, str], seen: list[tuple[str, str]]
) -> bool:
    """Whether `candidate` (from `prepare`) repeats anything already in `seen`."""
    cand_canon, cand_head = candidate
    if not cand_canon:
        return False
    for seen_canon, seen_head in seen:
        if cand_canon == seen_canon:
            return True
        if SequenceMatcher(None, cand_canon, seen_canon).ratio() < _SIMILARITY_FLOOR:
            continue
        if not _diff_is_significant(cand_head, seen_head):
            return True
    return False
