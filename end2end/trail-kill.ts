/**
 * Port-scoped TrailBase kill for Windows (global-setup.ts).
 *
 * Mirrors the posix branch: the process is matched by the command line it
 * was spawned with (`trail run --dev --address 127.0.0.1:<port>`), so a
 * parallel checkout running its own harness on another E2E_TRAILBASE_PORT
 * keeps its TrailBase. The previous blanket `taskkill /f /im trail.exe`
 * killed every TrailBase on the machine.
 *
 * Quoting: the payload travels as one double-quoted argument through
 * cmd.exe, so inner strings are single-quoted; WQL doubles a single quote
 * to embed one ('' → '). Get-CimInstance is the supported successor of
 * wmic (removed in Windows 11 24H2). A process that dies between listing
 * and stopping makes the pipeline exit non-zero — the caller treats any
 * failure the same way as "nothing to kill".
 */
export function winTrailKillCommand(port: number): string {
  return (
    `powershell -NoProfile -Command "Get-CimInstance Win32_Process ` +
    `-Filter 'Name=''trail.exe''' | ` +
    `Where-Object { $_.CommandLine -like '*--address 127.0.0.1:${port}*' } | ` +
    `ForEach-Object { Stop-Process -Id $_.ProcessId -Force }"`
  );
}
