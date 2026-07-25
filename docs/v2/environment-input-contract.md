# Explicit environment-input contract

`presolve environment --file <path>` reads only the caller-named dotenv file
and writes a schema-v1 manifest to standard output. It never reads ambient
process environment. Each non-comment line must be one unique `NAME=VALUE`
declaration; quoting, interpolation, and implicit loading are deliberately not
part of this compiler input contract.

Only names beginning with `PRESOLVE_PUBLIC_` enter `browserValues`. All other
valid environment names are server-owned and appear only in `serverValueNames`:
their values are omitted from the published artifact. Invalid names, duplicate
declarations, and NUL-bearing values fail closed.

This product establishes proven value admission and source provenance. A later
source-lowering product must bind authored environment reads to this manifest
before browser/server read diagnostics can claim complete coverage.
