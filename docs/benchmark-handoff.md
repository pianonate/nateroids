# Frame-rate benchmarking handoff

## Decision and limits

Keep the current release profile: optimization level 3, 12 codegen units,
`lto = false`, and no debug information. No substantial, repeatable FPS gain
has been established for a more expensive compiler configuration. If the FPS
difference is marginal, prefer shorter builds; substantial build-time increases
need substantial, consistent runtime improvements.

The 2026-09-06 experiments were interrupted by contention from other builds and
tests. Results below are observations, not a reliable ranking of compiler options.
The normal release executable was rebuilt afterward, and benchmark-only source
changes were not committed. The accepted renderer uses the one-pixel filtered
star footprint described in [Star Rendering](as-built/star-rendering.md).

## What was measured

Machine: Ryzen 9 9950X, RTX 5070 Ti, NVIDIA 595.71.05, Linux/NixOS, Rust 1.98.1.
The game used Vulkan, a 1280×720 window, Immediate presentation, and continuous
updates across focus changes.

The compiler comparison used active gameplay: physics, asteroid spawning,
collisions, star rotation, and twinkling were enabled. Timed inputs fired every
two seconds and briefly applied thrust and right turn every ten seconds. Each
configuration received three 30-second samples, with a restart and warm-up
between samples. The initial paused/static proposal was abandoned.

A temporary source copy seeded star generation with 20260906. Asteroid randomness
and time-driven simulation remained active, so these were comparable workloads,
not identical frame replays. FPS came from frame-count differences over monotonic
wall time. Diagnostic snapshots do not provide whole-run frame-time percentiles
or 1% lows, and application FPS is not the monitor's displayed frame rate.

### Initial pass

| Codegen units | LTO | FPS samples | Median FPS | Mean GPU utilization |
| ---: | --- | --- | ---: | ---: |
| 12 | false | 414.29, 416.63, 419.48 | 416.63 | 36.9% |
| 1 | false | 426.69, 422.86, 416.84 | 422.86 | 36.6% |
| 12 | thin | 355.76, 319.84, 306.85 | 319.84 | 31.3% |
| 1 | thin | 399.79, 404.17, 389.71 | 399.79 | 34.8% |
| 1 | fat | 411.37, 401.10, 420.71 | 411.37 | 37.3% |

Competing processes were not recorded throughout this pass. The apparent 1.5%
gain from one codegen unit is too small to justify a change on this evidence;
the apparent LTO regressions also need clean verification.

Observed variant build durations were 7m25s for one unit/no cross-crate LTO,
3m07s for 12 units/Thin LTO, 7m20s for one unit/Thin LTO, and 13m37s for one
unit/Fat LTO. Fat LTO spent roughly eight minutes on the final executable.
These builds had different cache histories and rebuilt dependencies. They are
not controlled clean-build comparisons or typical small-edit rebuild times.

### Contention and attempted clean repeat

Later baseline and one-unit medians fell to 227.74 and 272.62 FPS while unrelated
Rust builds/tests were active. Exclude these results from compiler comparisons.

After waiting for a quiet interval, the baseline measured 368.17, 351.22, and
355.46 FPS, with no monitored competing compiler/test processes detected. The
candidate then measured 378.97, 401.72, and 406.64 FPS, but `cargo-mend`/`rustc`
jobs appeared in its first two samples. The harness rejected that comparison.
Do not compare its median against the clean baseline or select its last sample
as evidence of a win. Process-name monitoring also cannot exclude every source
of machine load.

## Where the limit might be

The earlier, controlled investigation of the old star renderer isolated a real
CPU cost: removing per-frame per-star material updates raised one paused scene
from about 170 to 435 FPS. The shared material and shader twinkling address that
cost. Those numbers are not a before/after benchmark of current active gameplay.

Current GPU utilization around 30–40% suggests available GPU capacity, but does
not exclude driver waits, synchronization, or a GPU stage with poor utilization.
The old renderer barely responded to resolution changes; that test needs to be
repeated with the current renderer. The later CPU thread sample was contaminated,
and a fresh CPU profile of the current renderer was not completed. Scheduling,
render preparation, and driver submission are hypotheses, not established causes.

## Next run on a quiet machine

1. Pause build/test automation as well as manual builds. Reserve the whole test
   window; a quiet interval before launch was insufficient last time. Record
   background activity throughout each sample and discard affected samples.
2. Freeze the source revision, lockfile, and shader. Build
   every candidate before measuring. Start with the current profile versus one
   codegen unit, without cross-crate LTO. Revisit LTO only with a clean baseline.
3. Run actual gameplay with the same input sequence, resolution, presentation
   mode, and warm-up. Verify physics is unpaused and stars move and twinkle.
   Avoid manual camera changes or window resizing during a compiler comparison.
4. Alternate baseline/candidate/baseline/candidate instead of measuring all of
   one configuration first. Collect at least three 30–60-second samples per
   configuration. Record actor counts or use repeatable spawning/input playback
   to distinguish workload changes from compiler effects. Seeded stars alone
   do not make gameplay deterministic.
5. Compare the spread as well as the median. If a gain is small or inconsistent,
   keep the faster-building profile. Measure representative edit/rebuild times
   under matched cache conditions before adopting a slower profile.
6. In separate runs, compare 640×360, 1280×720, and 2560×1440, returning to the
   original size afterward. Hold gameplay workload as comparable as possible.
   A large resolution response points toward GPU rendering cost; a flat response
   motivates CPU/driver profiling but is not proof by itself.
7. Profile current gameplay separately from timing samples. Capture per-thread
   CPU time and sampled call stacks, distinguishing game systems, render
   preparation, driver work, and waits. If coordination dominates, a small
   Bevy worker-pool experiment is more relevant than further LTO tuning. Do not
   persist affinity or thread-count changes without an active-gameplay comparison.

### Build and launch the first two candidates

Run from the repository root. These overrides do not edit the release profile.
The camera and asset-loader dependencies resolve from the public hana mirror.

```sh
bench_dir=$(mktemp -d /tmp/nateroids-fps.XXXXXX)

cargo build --release --locked \
  --config 'profile.release.codegen-units=12' \
  --config 'profile.release.lto=false'
cp target/release/nateroids "$bench_dir/baseline"

cargo build --release --locked \
  --config 'profile.release.codegen-units=1' \
  --config 'profile.release.lto=false'
cp target/release/nateroids "$bench_dir/cgu1"

# Restore the ordinary executable before starting measurements.
cargo build --release --locked

# Launch one candidate at a time; close it before launching the other.
BEVY_ASSET_ROOT="$PWD" BRP_EXTRAS_PORT=15742 "$bench_dir/baseline"
BEVY_ASSET_ROOT="$PWD" BRP_EXTRAS_PORT=15742 "$bench_dir/cgu1"
```

For subsequent LTO experiments, use `--config 'profile.release.lto="thin"'`
or `--config 'profile.release.lto="fat"'` and save each executable separately.
Cargo's `lto = false` still permits local Thin LTO across a crate's codegen
units; it does not enable cross-crate LTO. See the
[Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html#lto).

### Measure application FPS through BRP

With a candidate running, finish the splash and warm up gameplay. In a second
terminal, this minimal sampler measures 30 seconds without changing game state:

```sh
python3 - <<'PY'
import json
import time
import urllib.request

def sample():
    body = json.dumps({"jsonrpc": "2.0", "id": 1,
                       "method": "brp_extras/get_diagnostics"}).encode()
    request = urllib.request.Request(
        "http://127.0.0.1:15742", data=body,
        headers={"Content-Type": "application/json"})
    before = time.monotonic()
    with urllib.request.urlopen(request, timeout=10) as response:
        result = json.load(response)["result"]
    return (before + time.monotonic()) / 2, result["frame_count"]

t0, f0 = sample()
time.sleep(30)
t1, f1 = sample()
print(f"{(f1 - f0) / (t1 - t0):.2f} FPS over {t1 - t0:.2f} seconds")
PY
```

This sampler does not automate inputs, detect competing work, or capture frame
percentiles. Keep those controls and observations alongside the result.

## Existing temporary artifacts

`/tmp/nateroids-release-tuning-20260906/` contains the source snapshot, saved
executables, build commands/logs, raw JSON samples, and Python harnesses.
`measure.py` drives gameplay and records competing compiler/test processes;
`clean_comparison.py` waits for a quiet interval but must still reject any run
where competing work restarts. Its last comparison failed that check.

The snapshot uses a sibling hana checkout and predates the public hana_lading
dependency update. Comparing its
Rust sources with the updated dependency found documentation-only differences,
but future comparisons should rebuild all candidates from one current tree.
Temporary artifacts can disappear when `/tmp` is cleaned; the commands above
are the durable starting point. Do not run the old harness unchanged and assume
it represents the latest source or guarantees a quiet machine.
