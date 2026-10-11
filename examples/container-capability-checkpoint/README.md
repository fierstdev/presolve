# OCI container checkpoint workload

This directory contains the real OCI workload used by `scripts/smoke-container-oci.sh`.

The image intentionally has no knowledge of the concrete KV provider selected by the environment. At runtime it receives only Presolve capability-bridge discovery variables:

- `PRESOLVE_CAPABILITY_ENDPOINT`
- `PRESOLVE_CAPABILITY_TOKEN`
- `PRESOLVE_CAPABILITY_PROTOCOL`

It calls the language-neutral `presolve-http-json/0.1` bridge and writes the key:

```text
presolve.chunk16.checkpoint = container-capability-ok
```

The host-side checkpoint harness verifies that value through the exact `KeyValueProvider` selected by the `DeploymentPlan`.

The local registry used by the smoke script exists only to give the test image a real OCI manifest descriptor and digest. Registry location is not written into the Application Contract or `.presolved` release.
