/** Flags that belong to other commands must not be silently ignored. */
export function assertCaptureFlags(
  argv: readonly string[],
  command: string | undefined,
): void {
  if (
    command !== "capture" ||
    !argv.some(
      (argument) => argument === "--run" || argument.startsWith("--run="),
    )
  ) {
    return;
  }
  throw new Error(
    "--run applies only to coverage and store; capture writes to --out",
  );
}
