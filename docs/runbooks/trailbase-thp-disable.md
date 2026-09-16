# TrailBase THP-disable launcher (`THP_DISABLE=1`)

**Status:** AVAILABLE (opt-in, disabled by default)
**Introduced:** PR with `origa_ui/thp_off.c`, container image `origa/ui`
**Related:** memory-usage investigation 2026-09-16 (Railway memory billing)

## Why

Railway hosts run with `transparent_hugepage=always`. The kernel backs the
mimalloc arenas of the (statically linked) `trail` binary with 2 MB huge
pages, and the cgroup memory accounting — which Railway's memory graph *and*
bill are based on — reports far above the actual working set.

A/B on identical containers (same image, empty DB, no traffic):

| Metric        | Baseline | `THP_DISABLE=1` | Delta  |
| ------------- | -------- | --------------- | ------ |
| cgroup memory | 944 MB   | 621 MB          | −34%   |
| `anon_thp`    | 792 MB   | 0 MB            | −100%  |
| boot → listen | ~3 s     | ~3 s            | noise  |

No functional regressions (auth, admin API, CRUD, static files, rate
limiting all verified identical).

## How it works

`thp_off` is a tiny static binary that calls `prctl(PR_SET_THP_DISABLE, 1)`
and then `exec`s the real command. The flag survives `fork(2)` and
`execve(2)` (see `PR_SET_THP_DISABLE(2const)`), so the launcher works even
though `trail` is static-pie linked and silently ignores `LD_PRELOAD`.

Fail-open on every level:

- unset / other value of the variable → plain `exec trail` as before;
- launcher binary missing → log line + plain `exec trail`;
- `prctl` fails (ancient kernel, restrictive seccomp) → warning to stderr,
  command runs anyway.

## Enable / disable / rollback

```bash
# enable (applied on the next deploy)
railway variables set --service origa-trailbase THP_DISABLE=1

# disable / rollback (applied on the next deploy)
railway variables set --service origa-trailbase THP_DISABLE=0
```

Then redeploy the service (or let the release pipeline's `deploy-to-railway`
job do it).

## Verify

Deploy logs must contain:

```text
[thp-off] pid=2 thp_disable: before=0 after=1
```

Inside the container, `grep anon_thp /sys/fs/cgroup/memory.stat` should stay
near zero (vs. hundreds of MB with THP enabled). The Railway memory graph
should drop by roughly a third.

## Trade-offs

Without THP the process gets 4 KB pages: slightly more TLB pressure and page
faults (potential slowdown for memory-bound code paths, e.g. the boot-time
WASM compilation), in exchange for honest memory accounting and a smaller
bill. Measured boot time was unchanged.
