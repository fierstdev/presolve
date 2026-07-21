# CLI Command Framework and Project Envelope

L9-B establishes stable command names and permanent exit-code values without claiming that every command is implemented. Command execution is introduced in later L9 slices only with its dedicated fixtures and documentation.

`load_explicit_project_envelope_v1` reads one caller-named public configuration file and decodes it through the strict L9-A codec. It does not search for `presolve.json`, inspect the current directory, enumerate source roots, glob, resolve symlinks, load a workspace manifest, or compile any source. The result is an explicit project envelope for the later request-construction layer.
