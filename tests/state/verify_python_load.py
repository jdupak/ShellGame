#!/usr/bin/env python3
"""Check that the real Python implementation accepts a state file written by the Rust binary.

Usage: verify_python_load.py STATE_DIR

Loads STATE_DIR/state.json with the real ``StateManager`` (migration included) and checks
that Python would write the very same bytes back. Byte-identical re-serialization also proves
that Python read every value the way the Rust side meant it.

Exit status 0 means accepted. Status 42 means Python rejected the file or would write
different bytes. Anything else (no pydantic, no Python sources, ...) means the environment is
broken, and callers such as tests/state_compat.rs should skip instead of failing.

Run it like the golden generator, from the repository root::

    PYTHONPATH=src uv run --no-project --with 'pydantic==2.12.5' python tests/state/verify_python_load.py DIR
"""

from __future__ import annotations

import os
import sys

from shellgame.state.manager import StateLoadError, StateManager

REJECTED = 42


def main(state_dir: str) -> int:
    # StateManager reads this when it is constructed, so set it first.
    os.environ["SHELLGAME_STATE_DIR"] = state_dir
    manager = StateManager()
    on_disk = manager.state_file.read_text(encoding="utf-8")

    try:
        state = manager.load()
    except StateLoadError as error:
        print(f"Python rejected the file: {error}")
        return REJECTED
    if state is None:
        print("Python found no state file")
        return REJECTED

    rewritten = state.model_dump_json(indent=2)
    if rewritten != on_disk:
        print("Python would write different bytes.")
        print("--- on disk ---")
        print(on_disk)
        print("--- Python ---")
        print(rewritten)
        return REJECTED

    print(f"Python accepted {manager.state_file} (current level {state.current_level})")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: verify_python_load.py STATE_DIR")
    sys.exit(main(sys.argv[1]))
