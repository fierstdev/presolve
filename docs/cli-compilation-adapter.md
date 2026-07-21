# CLI Compilation Adapter

L9-C accepts a complete caller-supplied configuration and ordered logical source inputs, derives the canonical candidate snapshot, and delegates exactly once to the L4 compiler service. It does not read source paths, discover projects, parse source, construct diagnostics, generate artifacts, or alter L5/L6 behavior.

The adapter is an internal command-layer seam. Public command parsing and authorized source loading remain later L9 work.
