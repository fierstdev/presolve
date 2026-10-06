# EdgeZero

**Build once. Run wherever your customer requires.**

EdgeZero is an application runtime and software-distribution platform for
software that must cross infrastructure boundaries.

An EdgeZero application declares what it needs rather than how its
infrastructure must be constructed.

The same application artifact can ultimately run across environments such as:

- local development machines
- SaaS infrastructure
- customer-controlled cloud accounts
- private VPCs
- on-premises infrastructure
- edge computers

## Status

EdgeZero is under active early development.

The current version is `0.0.1`.

No APIs, protocols, artifact formats, or interfaces should be considered
stable until explicitly documented as stable.

## Repository structure

```text
crates/
  ez-core/          Shared foundational types
  ez-contract/      Application Contract
  ez-runtime/       WebAssembly Component runtime
  ez-bundle/        EdgeZero application bundles
  ez-policy/        Deployment/security policy
  ez-resolver/      Capability resolution
  ez-provider-sdk/  Provider implementation SDK
  ez-node/          Managed/private runtime node
  ez-cli/           `ez` command-line interface

packages/
  sdk-typescript/       TypeScript guest SDK
  tooling-typescript/   TypeScript build tooling
  create-edgezero/      Project scaffolding

wit/                Versioned WIT interfaces
providers/          First-party capability providers
examples/           Example EdgeZero applications
tests/              Cross-crate integration fixtures
scripts/            Repository automation
docs/               Design and architecture documentation
