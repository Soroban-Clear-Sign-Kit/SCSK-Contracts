#!/usr/bin/env python3
import json
import os
import sys

def main():
    budget_file = "size-budget.json"
    if not os.path.exists(budget_file):
        print(f"Error: {budget_file} not found.", file=sys.stderr)
        sys.exit(1)

    with open(budget_file, "r") as f:
        budgets = json.load(f)

    all_passed = True
    print("Checking contract WASM size budgets...")

    for path, max_size in budgets.items():
        if not os.path.exists(path):
            print(f"FAIL: {path} not found! Did you run `stellar contract build`?", file=sys.stderr)
            all_passed = False
            continue

        actual_size = os.path.getsize(path)
        if actual_size > max_size:
            diff = actual_size - max_size
            print(f"FAIL: {path} exceeds size budget: {actual_size} bytes > {max_size} bytes (+{diff} bytes)", file=sys.stderr)
            all_passed = False
        else:
            remaining = max_size - actual_size
            pct = (actual_size / max_size) * 100
            print(f"PASS: {path} - {actual_size} / {max_size} bytes ({pct:.1f}%, {remaining} bytes remaining)")

    if not all_passed:
        print("\nOne or more contracts exceeded their size budget!", file=sys.stderr)
        sys.exit(1)

    print("\nAll contracts are within their size budget.")

if __name__ == "__main__":
    main()
