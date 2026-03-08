"""
Ghost API Python Threads Worker

Production-ready wrapper for threads-scraper that implements the GhostWorker interface
with direct cookie injection into Playwright.
"""

__version__ = "2.0.0"
__author__ = "Ghost API Team"

from .worker import (
    ThreadsWorker,
    GhostContext,
    PayloadBlob,
    WorkerHealth,
    execute_worker,
    get_worker_info,
    health_check,
)

__all__ = [
    "ThreadsWorker",
    "GhostContext",
    "PayloadBlob",
    "WorkerHealth",
    "execute_worker",
    "get_worker_info",
    "health_check",
]
