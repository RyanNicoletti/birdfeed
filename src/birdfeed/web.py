"""Flask application serving the aggregated feed and weekly summary."""

from __future__ import annotations

import logging

from flask import Flask, render_template

from . import config, db

log = logging.getLogger("birdfeed.web")


def create_app() -> Flask:
    app = Flask(__name__)
    db.init_db()

    @app.route("/")
    def index():
        days = db.articles_grouped_by_date(config.DISPLAY_WINDOW_DAYS)
        total = sum(len(arts) for _, arts in days)
        latest = db.latest_summary()
        summary_text, summary_range = (latest if latest else (None, None))
        return render_template(
            "index.html",
            days=days,
            total=total,
            window=config.DISPLAY_WINDOW_DAYS,
            summary_text=summary_text,
            summary_range=summary_range,
            source_count=len(config.SOURCES),
        )

    @app.route("/healthz")
    def healthz():
        return {"status": "ok"}

    return app
