# CLI Command Dispatch

L9-G makes `version` and `help` deterministic public commands. `version
--format json` emits the `presolve.cli-version` v1 document; `help` writes the
command list to stdout and exits successfully.

`create`, `dev`, `profile`, `watch`, `inspect`, `trace`, `benchmark`, and
`doctor` are recognized but reserved until a corresponding canonical L3-L8
product adapter exists. They write one structured tooling error to stderr and
exit `6`; no substitute compiler, server, profiler, graph, trace, or project
discovery behavior is invented.
