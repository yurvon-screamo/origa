/* Static launcher that disables Transparent Huge Pages for the exec'd process.
 *
 * Railway hosts run THP=always; the kernel then backs the mimalloc arenas of
 * the (statically linked) trail binary with 2 MB pages, which inflates the
 * cgroup memory accounting — and Railway's memory bill — far above the actual
 * working set (~+50% measured). prctl(PR_SET_THP_DISABLE) brings the reported
 * memory back to reality.
 *
 * Why a launcher instead of LD_PRELOAD: trail is static-pie linked, so the
 * dynamic loader never runs and LD_PRELOAD is silently ignored. The THP
 * disable flag survives fork(2) and execve(2) (see PR_SET_THP_DISABLE(2const)),
 * so setting it before exec is enough.
 *
 * Fail-open: if prctl fails (e.g. ancient kernel), the command still runs.
 */
#define _GNU_SOURCE
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/prctl.h>
#include <unistd.h>

#ifndef PR_SET_THP_DISABLE
#define PR_SET_THP_DISABLE 41
#endif
#ifndef PR_GET_THP_DISABLE
#define PR_GET_THP_DISABLE 42
#endif

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "[thp-off] usage: thp_off CMD [ARGS...]\n");
        return 2;
    }

    long before = prctl(PR_GET_THP_DISABLE, 0, 0, 0, 0);
    if (prctl(PR_SET_THP_DISABLE, 1, 0, 0, 0) != 0) {
        fprintf(stderr, "[thp-off] PR_SET_THP_DISABLE failed: %s (running anyway)\n",
                strerror(errno));
    } else {
        long after = prctl(PR_GET_THP_DISABLE, 0, 0, 0, 0);
        fprintf(stderr, "[thp-off] pid=%d thp_disable: before=%ld after=%ld\n",
                getpid(), before, after);
    }

    execvp(argv[1], &argv[1]);
    fprintf(stderr, "[thp-off] exec %s failed: %s\n", argv[1], strerror(errno));
    return 1;
}
