"""Birdfeed command-line interface.

Usage:
    birdfeed serve       Run the web server + background scheduler.
    birdfeed scrape      Run a single scrape now.
    birdfeed summarize   Generate this week's summary now.
    birdfeed init-db     Create the database schema.
"""

from __future__ import annotations

import argparse
import logging
import sys

from . import config


def _setup_logging() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s: %(message)s",
    )


def _cmd_serve(_args) -> int:
    from waitress import serve

    from .scheduler import start_scheduler
    from .web import create_app

    app = create_app()
    start_scheduler()
    logging.getLogger("birdfeed").info(
        "serving on http://%s:%d", config.HOST, config.PORT
    )
    serve(app, host=config.HOST, port=config.PORT)
    return 0


def _cmd_scrape(_args) -> int:
    from .scrape import run_scrape

    inserted = run_scrape()
    print(f"Inserted {inserted} new article(s).")
    return 0


def _cmd_summarize(_args) -> int:
    from .llm import generate_summary

    summary = generate_summary()
    print(summary)
    return 0


def _cmd_init_db(_args) -> int:
    from .db import init_db

    init_db()
    print(f"Initialized database at {config.DB_PATH}")
    return 0


def main(argv: list[str] | None = None) -> int:
    _setup_logging()
    parser = argparse.ArgumentParser(prog="birdfeed", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("serve", help="run web server + scheduler").set_defaults(func=_cmd_serve)
    sub.add_parser("scrape", help="run a single scrape now").set_defaults(func=_cmd_scrape)
    sub.add_parser("summarize", help="generate this week's summary").set_defaults(
        func=_cmd_summarize
    )
    sub.add_parser("init-db", help="create the database schema").set_defaults(
        func=_cmd_init_db
    )

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
