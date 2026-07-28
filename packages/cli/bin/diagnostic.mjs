export function unsupportedPlatformDiagnostic(version, platform, architecture) {
  return (
    `Presolve ${version} does not include a CLI binary for ${platform}-${architecture}. ` +
    "See https://github.com/fierstdev/presolve#supported-platforms."
  );
}
