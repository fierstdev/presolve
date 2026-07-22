/** Compares caller-owned canonical byte sequences without interpretation. */
export function equalCanonicalBytes(expected, actual) {
  if (!(expected instanceof Uint8Array) || !(actual instanceof Uint8Array)) return false;
  if (expected.byteLength !== actual.byteLength) return false;
  return expected.every((value, index) => value === actual[index]);
}

/** Creates immutable metadata for an already-declared local test command. */
export function declaredTest({ name, command, lane }) {
  if (!name || !command || !lane) throw new TypeError("declared test requires name, command, and lane");
  return Object.freeze({ name, command, lane });
}
