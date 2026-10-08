# Presolve

**Build once. Run wherever your customer requires.**

Presolve is an application runtime and software-distribution platform for
software that must cross infrastructure boundaries.

An Presolve application declares what it needs rather than how its
infrastructure must be constructed.

The same application artifact can ultimately run across environments such as:

- local development machines
- SaaS infrastructure
- customer-controlled cloud accounts
- private VPCs
- on-premises infrastructure
- edge computers

## Status

Presolve is under active early development.

The current version is `0.0.1`.

No APIs, protocols, artifact formats, or interfaces should be considered
stable until explicitly documented as stable.

## Repository structure

```text
crates/
  presolve-core/          Shared foundational types
  presolve-contract/      Application Contract
  presolve-runtime/       WebAssembly Component runtime
  presolve-bundle/        Presolve application bundles
  presolve-policy/        Deployment/security policy
  presolve-resolver/      Capability resolution
  presolve-provider-sdk/  Provider implementation SDK
  presolve-node/          Managed/private runtime node
  presolve-cli/           `presolve` command-line interface

packages/
  sdk-typescript/       TypeScript guest SDK
  tooling-typescript/   TypeScript build tooling
  create-presolve/      Project scaffolding

wit/                Versioned WIT interfaces
providers/          First-party capability providers
examples/           Example Presolve applications
tests/              Cross-crate integration fixtures
scripts/            Repository automation
docs/               Design and architecture documentation
