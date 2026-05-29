"""HTTP fetching.

curl_cffi is the primary client: it impersonates a real browser's TLS/JA3 and
HTTP-2 fingerprint, which is what lets us get past the bot protection that was
blocking the old reqwest-based scrapers. A urllib fallback keeps the app working
if curl_cffi is ever unavailable on the host.
"""

from __future__ import annotations

import logging

log = logging.getLogger("birdfeed.http")

# The browser profile curl_cffi impersonates. "chrome" tracks a recent stable
# Chrome fingerprint shipped with the installed curl_cffi version.
IMPERSONATE = "chrome"
DEFAULT_TIMEOUT = 25

_BROWSER_HEADERS = {
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9",
}

try:
    from curl_cffi import requests as _cffi  # type: ignore

    _HAS_CFFI = True
except Exception as exc:  # pragma: no cover - only hit when dependency missing
    _HAS_CFFI = False
    log.warning("curl_cffi unavailable (%s); falling back to urllib", exc)


class FetchError(Exception):
    """Raised when a URL cannot be fetched successfully."""


def _fetch_urllib(url: str, timeout: int) -> bytes:
    import urllib.request

    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) "
                "Gecko/20100101 Firefox/120.0"
            ),
            **_BROWSER_HEADERS,
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.read()
    except Exception as exc:  # noqa: BLE001
        raise FetchError(f"urllib fetch failed for {url}: {exc}") from exc


def get_bytes(url: str, timeout: int = DEFAULT_TIMEOUT) -> bytes:
    """Fetch a URL and return the raw response body, following redirects."""
    if _HAS_CFFI:
        try:
            resp = _cffi.get(
                url,
                impersonate=IMPERSONATE,
                timeout=timeout,
                allow_redirects=True,
                headers=_BROWSER_HEADERS,
            )
            if resp.status_code >= 400:
                raise FetchError(f"HTTP {resp.status_code} from {url}")
            return resp.content
        except FetchError:
            raise
        except Exception as exc:  # noqa: BLE001 - network/curl errors are varied
            log.warning("curl_cffi failed for %s (%s); trying urllib", url, exc)
            return _fetch_urllib(url, timeout)
    return _fetch_urllib(url, timeout)


def get_text(url: str, timeout: int = DEFAULT_TIMEOUT) -> str:
    """Fetch a URL and decode it as text (UTF-8, lenient)."""
    return get_bytes(url, timeout).decode("utf-8", errors="replace")


def resolve_final_url(url: str, timeout: int = DEFAULT_TIMEOUT) -> str:
    """Follow redirects and return the final landing URL (best effort)."""
    if _HAS_CFFI:
        try:
            resp = _cffi.get(
                url,
                impersonate=IMPERSONATE,
                timeout=timeout,
                allow_redirects=True,
                headers=_BROWSER_HEADERS,
            )
            return str(resp.url)
        except Exception as exc:  # noqa: BLE001
            log.debug("resolve_final_url failed for %s: %s", url, exc)
    return url
