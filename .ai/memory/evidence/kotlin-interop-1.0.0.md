# Kotlin companion interop — re-run on the 1.0.0 tree

Date: 2026-10-28 (primo, JDK 21, Gradle wrapper 8.14.4, offline).
Server binary: the primo in-tree 1.0.0 build
(`ardor/target/debug/ardor`, copied to `/tmp/ardor-bin-1.0.0`
for the run; `ARDOR_BIN` env, fixture defaulting to
`ardor-companion/fixtures/server-home`).

## Result: 1/1 PASS

`ArdorServerInteropTest` — "kotlin client drives the rust
server through a full session": **tests=1, skipped=0,
failures=0, errors=0** (JUnit XML,
`ardor-companion/client/build/test-results/test/`).

The session, as at 0.1.0: `server/discover` →
supportedVersions `["2026-07-28"]`; `tools/list` = 46 tools;
`get_project`; `run_playtest` seed 7 → hash `14b271c2…`
(agreement with the server's recorded run); `publish_preflight`
itch_io all-checks-ok; sent methods contain **no `initialize`**
(legacy handshake never sent).

## Companion unit battery (same run)

`:qa:test` — AuditReadinessTest 14/14 + ReplayTest 8/8 =
**22/22 passed** in each of the debug and release variants,
0 skipped, 0 failures.

First invocation note (honesty): the first `:client:test` run
reported the interop test **skipped=1** because `ARDOR_BIN` was
passed as a Gradle property instead of the environment
variable the build reads; the run above corrects that and the
test genuinely executed against the 1.0.0 binary.
