# ardor — benchmarks

Measured 2026-11-27 on primo (13th Gen Intel Core i7-13620H, 16 threads,
62 GiB RAM, cargo 1.100.0). Primo was under multi-lane load that night
(load average in the tens), so these are baselines, not peaks.

This file lives in the `ardor/` crate directory, beside the crate it
measures. The crate's pre-existing example
(`examples/canon_census_fixture.rs`) is untouched.

## Workloads (`benches/hot_paths.rs`, hand-rolled on `std::time`)

Sessions run against a throwaway server home (`--home <scratch>`).

- `cold_start_discover`: spawn the server, send `server/discover`, await
  the response; 10 spawns.
- `tools_list`: `tools/list` over one warm stdio session; 20 warmup +
  200 measured requests.
- `tool_call_list_projects`: `tools/call list_projects {}` (read-only)
  over the same warm session; 20 warmup (mixed with tools/list) + 200
  measured requests.

## Results

| Workload | µs/op | ops/s |
|---|---|---|
| cold_start_discover | 7,806.3 | 128 |
| tools_list | 1,678.8 | 596 |
| tool_call_list_projects | 559.3 | 1,788 |

## Example

`examples/stdio_session.rs` drives one modern-era session (discover →
tools/list → `list_projects`): server ardor 1.1.0, 61 tools,
`list_projects` isError=false with the empty project list of a fresh
scratch home.

## Gates (2026-11-27)

- `cargo build --examples` clean; example run green.
- `cargo bench --bench hot_paths` ran to the numbers above.
- `cargo test -p ardor`: 108 passed / 0 failed.
- `cargo clippy` on the added files: no warnings in the added files
  (the only clippy output is the pre-existing cargo manifest warning
  `package.homepage is redundant with package.repository`, untouched).
  rustfmt applied to the added files only.
