"""Entry point for ShellGame CLI."""

import sys
from shellgame.cli.commands import cli

if __name__ == "__main__":
    sys.exit(cli())
