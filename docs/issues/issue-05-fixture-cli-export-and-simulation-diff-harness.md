# Issue 05: Fixture Integrity Verification and Scenario Inspection CLI (C7)

## Description
The test fixture suite in `fixtures/` contains versioned, reproducible scenario files alongside an integrity manifest (`fixtures/manifest.json`). Currently, verifying manifest checksums and regenerating fixtures requires running the Rust test runner:
```bash
cargo test --package fixture generate_and_verify_reproducible_fixtures
```

External contributors, TypeScript developers, and hardware wallet engineers integrating SCSK often work outside of Rust test harnesses. A lightweight, standalone CLI utility (in Python, matching the existing `scripts/check-size-budget.py`) that can independently inspect scenario files and verify manifest SHA-256 checksums without compiling the full test suite will streamline cross-language integration and debugging.

## Complexity
`Trivial`

## Authorization & State Changes
- **Who authorizes**: Developer tooling / read-only utility; no smart contract state mutations or network transactions are executed.

## Requirements
1. Create a standalone script `scripts/verify-fixtures.py` that requires standard Python 3 with zero external package dependencies.
2. The script must support two primary operations:
   - `--check`: Iterates through all files indexed in `fixtures/manifest.json`, calculates their SHA-256 hashes, and asserts that they match the committed manifest checksums. Exits with non-zero status on any mismatch.
   - `--inspect <scenario_id>`: Loads `fixtures/<scenario_id>.json` and outputs a formatted terminal summary displaying scenario name, description, contract functions invoked, root authorization address, and event count.
3. Add a CI check step or hook verifying that `python scripts/verify-fixtures.py --check` runs and exits cleanly.
4. Document the utility and usage examples in `fixtures/README.md` and `ARCHITECTURE.md`.
5. Require `cargo test` and `stellar contract build`.

## Files to Touch
- `scripts/verify-fixtures.py`
- `fixtures/README.md`
- `ARCHITECTURE.md`
- `.github/workflows/ci.yml` (optional CI step)

## Suggested Execution
1. Create `scripts/verify-fixtures.py` using Python's built-in `hashlib`, `json`, `argparse`, and `pathlib` modules.
2. In `--check` mode, load `fixtures/manifest.json`, read each file in binary mode (`open(..., 'rb')`), calculate `hashlib.sha256().hexdigest()`, and compare with `entry["sha256"]`.
3. In `--inspect` mode, read the scenario JSON, print the header, parse `auth_entries` to display function names and addresses, and summarize `expected_events`.
4. Test the script locally:
   ```bash
   python scripts/verify-fixtures.py --check
   python scripts/verify-fixtures.py --inspect 01_standard_require_auth
   ```
5. Update `fixtures/README.md` with command examples.
6. Verify `cargo test` and `stellar contract build` pass.

## Acceptance Criteria
- [ ] `scripts/verify-fixtures.py` is implemented using Python 3 standard library with `--check` and `--inspect` modes.
- [ ] Running `python scripts/verify-fixtures.py --check` passes against the committed `fixtures/manifest.json`.
- [ ] Tampering with any scenario file causes `python scripts/verify-fixtures.py --check` to report a mismatch and exit with non-zero code.
- [ ] Usage instructions are documented in `fixtures/README.md`.
- [ ] `cargo test` and `stellar contract build` remain fully functional.
