#!/usr/bin/env python3
"""Distribute backlog tasks across a team roster.

The default mode assigns the full backlog with unlimited capacity. Use
--capacity for sprint planning. Dependencies are preserved as metadata; the
producer should still review blocked tasks before starting a sprint.
"""
from __future__ import annotations

import argparse
import csv
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PRIORITY = {"P0": 0, "P1": 1, "P2": 2}


def load_csv(path: Path):
    with path.open(newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tasks", default=str(ROOT / "tasks/tasks.csv"))
    parser.add_argument("--team", default=str(ROOT / "tasks/team.example.csv"))
    parser.add_argument("--output", default=str(ROOT / "tasks/assignments.csv"))
    parser.add_argument("--capacity", type=float, default=0.0,
                        help="Per-person capacity in days; 0 means unlimited.")
    parser.add_argument("--status", default="todo",
                        help="Only assign tasks with this status.")
    args = parser.parse_args()

    tasks = load_csv(Path(args.tasks))
    team = load_csv(Path(args.team))
    if not team:
        raise SystemExit("team roster is empty")

    load = {member["name"]: 0.0 for member in team}
    assigned = []
    unassigned = []

    ordered = sorted(
        (t for t in tasks if t.get("status", "todo") == args.status),
        key=lambda t: (PRIORITY.get(t["priority"], 9), t["phase"], t["id"]),
    )

    for task in ordered:
        estimate = float(task.get("estimate_days", "0") or 0)
        role = task.get("role", "")
        squad = task.get("squad", "")

        candidates = [m for m in team if m.get("role") == role]
        if not candidates:
            candidates = [m for m in team if m.get("squad") == squad]
        if not candidates:
            candidates = team

        if args.capacity > 0:
            candidates = [m for m in candidates if load[m["name"]] + estimate <= args.capacity]

        if not candidates:
            unassigned.append(task)
            continue

        member = min(candidates, key=lambda m: (load[m["name"]], m["name"]))
        row = dict(task)
        row["owner"] = member["name"]
        assigned.append(row)
        load[member["name"]] += estimate

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = list(tasks[0].keys()) if tasks else []
    if "owner" not in fieldnames:
        fieldnames.append("owner")
    with output.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(assigned)

    board = output.with_suffix(".md")
    by_owner = defaultdict(list)
    for row in assigned:
        by_owner[row["owner"]].append(row)
    lines = ["# Task assignments", "", f"Assigned: {len(assigned)}", f"Unassigned: {len(unassigned)}", ""]
    for owner in sorted(by_owner):
        days = sum(float(r.get("estimate_days", "0") or 0) for r in by_owner[owner])
        lines += [f"## {owner} — {days:g} days", ""]
        for row in by_owner[owner]:
            lines.append(f"- [ ] **{row['id']}** {row['title']} (`{row['priority']}`, {row['estimate_days']}d)")
        lines.append("")
    if unassigned:
        lines += ["## Unassigned", ""]
        for row in unassigned:
            lines.append(f"- [ ] **{row['id']}** {row['title']} (`{row['priority']}`, {row['estimate_days']}d)")
    board.write_text("\n".join(lines), encoding="utf-8")

    print(f"Assigned {len(assigned)} tasks; unassigned {len(unassigned)}")
    for name in sorted(load):
        print(f"{name}: {load[name]:g} days")


if __name__ == "__main__":
    main()
