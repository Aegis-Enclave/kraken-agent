import io
from typing import Any

import psutil
from rich.panel import Panel
from rich.text import Text
import inspect

from tui_gateway import slash_worker


class _FakeSlashCli:
    console: Any

    def __init__(self):
        self.console = None

    def process_command(self, command):
        self.console.print(Panel(Text("Previous Conversation"), title="Resume"))


def test_run_strips_rich_color_escapes_from_slash_output():
    cli_obj = _FakeSlashCli()

    output = slash_worker._run(cli_obj, "/resume")  # type: ignore[arg-type]

    assert "\x1b[" not in output
    assert "Previous Conversation" in output
    assert "Resume" in output


def test_is_orphaned_true_when_ppid_changes():
    # Our parent went away and we were reparented to a subreaper/init.
    assert slash_worker._is_orphaned(1234, getppid=lambda: 999999) is True


def test_is_orphaned_false_when_direct_parent_is_unchanged():
    original_ppid = 1234
    assert slash_worker._is_orphaned(original_ppid, getppid=lambda: original_ppid) is False


def test_parent_death_watchdog_contract_has_no_create_time_plumbing():
    assert list(inspect.signature(slash_worker._is_orphaned).parameters) == [
        "original_ppid",
        "getppid",
    ]
    assert list(inspect.signature(slash_worker._start_parent_death_watchdog).parameters) == [
        "original_ppid",
    ]
