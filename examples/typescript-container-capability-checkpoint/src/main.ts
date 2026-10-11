import { kv } from "@presolve/sdk";
import { installContainerBridge } from "@presolve/sdk/container";

const PROOF_KEY = "presolve.chunk16.checkpoint";
const PROOF_VALUE = "container-capability-ok";

const restore = installContainerBridge();

try {
  await kv.set(PROOF_KEY, PROOF_VALUE);
  console.log(`wrote ${PROOF_KEY}=${PROOF_VALUE} through @presolve/sdk`);
} finally {
  restore();
}
