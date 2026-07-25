import { platform } from "node:process";
import { spawnSync } from "node:child_process";

export function runNpm(arguments_, options = {}) {
  const npm = platform === "win32" ? (process.env.ComSpec ?? "cmd.exe") : "npm";
  const npmArguments =
    platform === "win32"
      ? ["/d", "/s", "/c", "npm.cmd", ...arguments_]
      : arguments_;

  return spawnSync(npm, npmArguments, {
    encoding: "utf8",
    ...options,
  });
}

