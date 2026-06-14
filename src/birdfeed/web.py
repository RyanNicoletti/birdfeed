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
        window = config.DISPLAY_WINDOW_DAYS
        bird_flu_days = db.articles_grouped_by_date(window, category="bird_flu")
        other_days = db.articles_grouped_by_date(window, category="other")
        bird_flu_total = sum(len(arts) for _, arts in bird_flu_days)
        other_total = sum(len(arts) for _, arts in other_days)
        latest = db.latest_summary()
        summary_text, summary_range = (latest if latest else (None, None))
        return render_template(
            "index.html",
            bird_flu_days=bird_flu_days,
            other_days=other_days,
            bird_flu_total=bird_flu_total,
            other_total=other_total,
            window=window,
            summary_text=summary_text,
            summary_range=summary_range,
            source_count=len(config.SOURCES),
        )

    @app.route("/healthz")
    def healthz():
        return {"status": "ok"}

    return app
