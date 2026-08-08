import type { StandardSchemaV1 } from "presolve";

export const displayNameSchema = {
  "~standard": {
    version: 1,
    vendor: "presolve-test",
    validate(value: unknown) {
      return typeof value === "string" && value.length >= 2
        ? { value }
        : { issues: [{ message: "Use at least two characters." }] };
    },
    types: undefined as unknown as {
      readonly input: string;
      readonly output: string;
    },
  },
} satisfies StandardSchemaV1<string>;

export const lookalikeSchema = {
  "~standard": {
    version: 2,
    vendor: "not-standard-schema-v1",
    validate(value: unknown) {
      return { value };
    },
  },
};

export async function saveProfile(
  value: unknown,
  signal: AbortSignal,
): Promise<void> {
  await Promise.resolve({ value, signal });
}

export async function saveServerProfile(
  formData: FormData,
  signal: AbortSignal,
): Promise<{ saved: boolean }> {
  return { saved: formData.has("name") && !signal.aborted };
}
