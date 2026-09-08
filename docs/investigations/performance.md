# Frame-rate investigation

Investigated September 2026 on a Ryzen 9 9950X (16 cores, 32 logical CPUs),
RTX 5070 Ti, NixOS, NVIDIA 595.71.05, Rust 1.98.1, and Bevy 0.19.1.

## Findings and recommendation

Keep the release profile at optimization level 3, 12 codegen units, no
cross-crate LTO, and no debug information. Compiler experiments did not
establish a substantial, repeatable FPS improvement. Small runtime gains do
not justify substantially longer builds.

Reducing Bevy's compute pool was more effective: two workers improved median
FPS by 27% while reducing total CPU usage by about 45%. Start with an
application-scoped two-worker candidate; compare four workers in heavier
scenes before choosing a lasting policy. Runtime tuning remained experimental:
no worker, scheduler, affinity, or NixOS setting was adopted during this
investigation. The normal release executable was restored afterward.

The renderer already uses shared materials and shader twinkling with a
one-pixel filtered star footprint; see [Star Rendering](../as-built/star-rendering.md).

## Measurement conditions

The game ran through Vulkan at 1280×720 with Immediate presentation and
continuous updates across focus changes. Physics, asteroid spawning,
collisions, star rotation, and twinkling remained active. Scripted inputs fired
every two seconds and applied brief thrust/right turn every ten seconds.

Comparisons used fresh launches, approximately 13 seconds of warm-up, and
three 30-second samples per configuration in alternating orders. Stars used a
fixed seed; asteroids and simulation timing remained nondeterministic. Worker
runs recorded actor counts: typically five asteroids growing to 15–19, with
one spaceship and active firing. FPS is frame-count growth over monotonic wall
time, not displayed monitor refresh rate. Full frame-time distributions and
1% lows were not captured.

No monitored compiler/test jobs appeared during the retained comparisons.
Process-name checks cannot exclude all contention. Desktop indexing affected
parts of the compiler comparison; contaminated earlier build/test runs were
excluded. Compare within each series, not across their different baselines.

## Compiler settings

The compiler series reused saved executables after checking game-source and
shader equivalence apart from the star seed. It predates the public hana
dependency update; relevant dependency source comparisons found no functional
change. Worker tests rebuilt from the public-dependency source and lockfile,
then varied runtime settings in one executable with a fixed release profile.

| Codegen units | LTO | Median FPS | Sample range |
| ---: | --- | ---: | --- |
| 12 | false | 404 | 373–412 |
| 1 | false | 380 | 375–406 |
| 12 | thin | 376 | 355–380 |
| 1 | thin | 396 | 394–419 |
| 1 | fat | 402 | 392–425 |

No candidate beat the baseline in all three rounds. The spread and desktop
activity prevent a precise ranking or a claim that LTO inherently regresses
performance. Observed variant builds took roughly 3–14 minutes, but different
cache histories make these unsuitable for controlled build-time comparisons.

## Compute workers

Bevy allocated 24 compute, four async, and four I/O workers by default. Only
the compute pool changed in this comparison. CPU usage is summed process CPU
time expressed in cores, not a count of occupied physical cores.

| Compute workers | Median FPS | Sample range | Gain | Mean CPU cores used |
| ---: | ---: | --- | ---: | ---: |
| 24 (default) | 415 | 410–418 | — | 3.30 |
| 16 | 436 | 432–452 | 5% | 2.96 |
| 12 | 452 | 451–457 | 9% | 2.80 |
| 8 | 478 | 472–482 | 15% | 2.55 |
| 4 | 507 | 505–515 | 22% | 2.17 |
| 2 | 526 | 524–532 | 27% | 1.82 |

Two-worker samples were follow-ups alternating with the scheduling/affinity
candidates below. Return controls measured 406 FPS for default and 523 for four
workers. One worker reached only 457 FPS in a single exploratory sample.

### Scheduling and CPU placement

Serial scheduling used `SingleThreadedExecutor` for `PreUpdate`, `Update`,
`PostUpdate`, and `FixedUpdate`. Rendering, nested physics schedules, and
internal parallel queries retained their existing behavior. Affinity confined
the benchmark process to CPUs `0-7,16-23`: eight physical cores and their SMT
siblings sharing one L3 cache on this machine. Async and I/O pools stayed at
four workers each.

| Configuration | Median FPS | Sample range |
| --- | ---: | --- |
| 4 workers, serial update stages | 544 | 531–546 |
| 2 workers, serial update stages | 542 | 530–558 |
| 4 workers, one L3 cache group | 561 | 547–562 |
| 2 workers, serial stages and one L3 cache group | 554 | 549–559 |

These are three-sample results. Serial scheduling with the default pool reached
only 432 FPS in one exploratory sample. The pool reduction supplied most of
the gain; combining settings did not outperform the simpler affinity case.
Affinity changes cache locality, migration, and scheduling opportunities, so
it does not isolate cross-cache traffic. Its CPU IDs are machine-specific.

## CPU evidence and remaining limits

Separate 15-second user-cycle profiles compared default and two workers with
normal scheduling and unrestricted CPU placement. Profiled FPS was excluded
from the timing comparisons.

| Function self-cycle share | Default | Two workers |
| --- | ---: | ---: |
| Contended mutex lock | 7.28% | 1.14% |
| Bounded runnable queue pop | 5.68% | 0.60% |
| Unbounded runnable queue pop | 1.62% | 1.04% |

The percentages have different total-cycle denominators and do not measure
blocked wall time. Together with higher FPS and lower total CPU usage, they
support excess worker coordination as a material cost. Remaining samples were
spread across scheduling, memory copies/allocation, visibility, and render
preparation; no single game function dominated.

A separate resolution sweep measured 399 FPS at 640×360, 392 at 2560×1440,
and 416 on returning to 1280×720. Mean GPU utilization was 31%, 72%, and 40%.
These single samples suggest pixel throughput was not the primary limit,
but cannot rule out GPU/driver synchronization. Resizing during startup hit a
multisampled depth-texture validation failure; changing the full resolution
after warm-up succeeded.

## Next validation

- Compare two and four workers against default in heavier gameplay scenes,
  recording full frame-time distributions and actor counts. Test the MacBook
  separately before generalizing a worker policy.
- Freeze source, lockfile, and assets; finish all builds before timing. Alternate
  configurations, monitor background activity throughout, and use repeatable
  spawning/input playback to reduce workload variation.
- Trace frame/wait timing and allocation call sites after reducing the pool.
  Aggregate CPU sample shares alone do not identify the frame's critical path.
- Keep any adopted policy scoped to nateroids. Validate serial scheduling
  separately; do not turn this machine's affinity mask into a global default.
