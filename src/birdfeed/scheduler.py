"""Background scheduling of the daily scrape and weekly summary."""

from __future__ import annotations

import logging
import os

from apscheduler.schedulers.background import BackgroundScheduler
from apscheduler.triggers.cron import CronTrigger

from . import llm, scrape

log = logging.getLogger("birdfeed.scheduler")

# Optional timezone override (e.g. "America/New_York"); defaults to system local.
_TZ = os.getenv("BIRDFEED_TZ") or None


def _scrape_job() -> None:
    try:
        scrape.run_scrape()
    except Exception:  # noqa: BLE001
        log.exception("scheduled scrape failed")


def _summary_job() -> None:
    try:
        llm.generate_summary()
    except Exception:  # noqa: BLE001
        log.exception("scheduled summary failed")


def start_scheduler() -> BackgroundScheduler:
    """Start the in-process scheduler.

    The scrape runs once a day, late in the evening, so a same-day publish
    filter still captures the full day's articles. The weekly summary runs
    Monday morning over the trailing seven days.
    """
    sched = BackgroundScheduler(timezone=_TZ) if _TZ else BackgroundScheduler()
    sched.add_job(
        _scrape_job,
        CronTrigger(hour=23, minute=0),
        id="daily_scrape",
        replace_existing=True,
        misfire_grace_time=3600,
    )
    sched.add_job(
        _summary_job,
        CronTrigger(day_of_week="mon", hour=8, minute=0),
        id="weekly_summary",
        replace_existing=True,
        misfire_grace_time=3600,
    )
    sched.start()
    log.info("scheduler started (daily scrape 23:00, weekly summary Mon 08:00)")
    return sched
