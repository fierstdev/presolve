import { kv } from "@presolve/sdk";

const CHECKPOINT_KEY = "presolve.chunk19.typescript-component";

export async function run(input: string): Promise<string> {
  await kv.set(CHECKPOINT_KEY, input);
  return (await kv.get(CHECKPOINT_KEY)) ?? "missing";
}
