# CLI Command Dispatch

L9-G makes `version` and `help` deterministic public commands. `version
--format json` emits the `presolve.cli-version` v1 document; `help` writes the
command list to stdout and exits successfully.

`create`, `dev`, `benchmark`, and `doctor` remain recognized reserved commands:
they write one structured tooling error to stderr and exit `6`. `watch --once`
is active only through its explicit L8 replacement-candidate adapter, while
`inspect`, `graph`, `trace`, and `profile` are active only through their L11
strict named-product readers. No substitute compiler, server, profiler, graph,
trace, or project-discovery behavior is invented.
