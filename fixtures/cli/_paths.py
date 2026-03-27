"""Repository root resolution for `fixtures.cli`."""

from __future__ import annotations

from pathlib import Path

# fixtures/cli/_paths.py -> fixtures -> repo root
REPO_ROOT = Path(__file__).resolve().parent.parent.parent
