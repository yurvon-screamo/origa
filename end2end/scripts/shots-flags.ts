/** Flags that belong to other commands must not be silently ignored. */
export function assertCaptureFlags(argv: readonly string[]): void {
  if (
    process.argv[2] !== "capture" ||
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
