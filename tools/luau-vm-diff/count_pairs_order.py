#!/usr/bin/env python3
"""Counts disagreeing tables in the generated pairs-order corpus.

`make luau-vm-diff` counts disagreements per case and profile; a generated
case holds many tables (gen_pairs_order_cases.py), so this splits the full
outputs `--verbose` prints for a run that differs from its snapshot by table
label and counts (table, profile) pairs whose lines differ.

    make luau-vm-diff LUAU_VM_DIFF_FLAGS="--filter pairs_order_gen --verbose" > out.txt
    python3 tools/luau-vm-diff/count_pairs_order.py out.txt
"""

import collections
import json
import re
import sys


def parse(debug):
    """A Rust `{:?}` of a `Vec<String>`, as a Python list."""
    debug = re.sub(r"\\u\{([0-9a-f]+)\}", lambda m: "\\u%04x" % int(m.group(1), 16), debug)
    return json.loads(debug)


def by_table(lines):
    """Output lines grouped by table label (`T<file>.<n>`)."""
    groups = collections.defaultdict(list)
    for line in lines:
        label = line.replace("\\t", "\t").split("\t")[0].split(" ")[0]
        table = label.rsplit(".", 1)[0]
        if table.startswith("T"):
            groups[table].append(line)
    return groups


def main():
    current = None
    reference = None
    bad = set()
    profiles = set()
    for line in open(sys.argv[1], encoding="utf-8"):
        match = re.match(r"(FAIL|ok  ) (\S+) \[(\S+)\]", line)
        if match:
            current = match.group(3)
            profiles.add(current)
        elif line.startswith("  full output snapshot: "):
            reference = by_table(parse(line[len("  full output snapshot: "):]))
        elif line.startswith("  full output nuxie: "):
            nuxie = by_table(parse(line[len("  full output nuxie: "):]))
            for table in set(reference) | set(nuxie):
                if reference.get(table) != nuxie.get(table):
                    bad.add((table, current))
    print("disagreeing (table, profile) pairs:", len(bad))
    print("distinct tables disagreeing:", len({table for table, _ in bad}))
    for profile in sorted(profiles):
        print(" ", profile, sum(1 for _, p in bad if p == profile))


if __name__ == "__main__":
    main()
