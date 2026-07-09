/**
 * Placeholder runtime package.
 *
 * The runtime must remain subordinate to compiler output. Do not add general
 * framework behavior here until a fixture proves the compiler requires it.
 */
export type ResumeManifest = {
  version: 1;
  components: Array<{
    id: string;
    tagName: string;
  }>;
};

export function version(): string {
  return "0.0.0";
}
