#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shlex
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def iso_utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_log(
    log_path: Path,
    *,
    timestamp: str,
    executor: str,
    purpose: str,
    commands: str,
    result: str,
    stdout: str | None = None,
    stderr: str | None = None,
    exit_code: int | None = None,
) -> None:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("a", encoding="utf-8") as f:
        f.write("- Timestamp (UTC): " + timestamp + "\n")
        f.write("- Executor: " + executor + "\n")
        f.write("- Command(s): " + commands.strip() + "\n")
        f.write("- Purpose: " + purpose.strip() + "\n")
        if exit_code is not None:
            f.write(f"- Exit code: {exit_code}\n")
        f.write("- Result: " + result + "\n")
        if stdout:
            f.write("- Stdout (truncated):\n")
            f.write("```\n")
            f.write(_truncate(stdout))
            f.write("\n````\n")
        if stderr:
            f.write("- Stderr (truncated):\n")
            f.write("```\n")
            f.write(_truncate(stderr))
            f.write("\n````\n")
        f.write("\n")


def _truncate(s: str, limit: int = 2000) -> str:
    if len(s) <= limit:
        return s
    head = s[: limit - 20]
    return head + "\n...[truncated]..."


def run_and_log(args: argparse.Namespace) -> int:
    ts = iso_utc_now()
    log_file = Path(os.getenv("GCP_ACTIVITY_LOG_FILE", "gcp-activity-log.md"))
    executor = args.executor or os.getenv("GCP_LOG_EXECUTOR", "Agent")
    purpose = args.purpose or "Unspecified"
    if args.run:
        # Execute the command in a login shell for compatibility
        cmd = args.run
        # Show a normalized form in the log for readability
        display_cmd = cmd
        try:
            proc = subprocess.run(cmd, shell=True, capture_output=True, text=True)
            rc = proc.returncode
            result = "success" if rc == 0 else "error"
            write_log(
                log_file,
                timestamp=ts,
                executor=executor,
                purpose=purpose,
                commands=display_cmd,
                result=result,
                stdout=proc.stdout.strip(),
                stderr=proc.stderr.strip(),
                exit_code=rc,
            )
            return rc
        except Exception as e:  # pragma: no cover
            write_log(
                log_file,
                timestamp=ts,
                executor=executor,
                purpose=purpose,
                commands=display_cmd,
                result="error",
                stdout="",
                stderr=str(e),
                exit_code=-1,
            )
            return 1
    else:
        # Append-only mode (no command execution)
        result = args.result or "success"
        commands = args.commands or "n/a"
        details = args.details or ""
        write_log(
            log_file,
            timestamp=ts,
            executor=executor,
            purpose=purpose,
            commands=commands,
            result=result,
            stdout=details,
            stderr=None,
            exit_code=None,
        )
        return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Append structured entries to gcp-activity-log.md")
    mode = p.add_mutually_exclusive_group(required=False)
    mode.add_argument("--run", help="Shell command to execute and log result")
    p.add_argument("--executor", help="Executor label (User/Agent/CI)")
    p.add_argument("--purpose", help="Purpose of the action")
    # Append-only fields
    p.add_argument("--commands", help="Commands text when not using --run")
    p.add_argument("--result", choices=["success", "error"], help="Result when not using --run")
    p.add_argument("--details", help="Additional details text for append-only mode")
    return p


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return run_and_log(args)


if __name__ == "__main__":
    sys.exit(main())

