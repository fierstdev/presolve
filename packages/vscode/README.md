# Presolve for Visual Studio Code

Install this extension and open a project created with `pnpm create presolve`.
It verifies the configured project root and uses the project-local Presolve
compiler for:

- exact diagnostics on save or on demand;
- compiler-derived component CodeLens and source explanation;
- workspace checks, production builds, and doctor output;
- release-train and workspace status.

Presolve preserves ordinary TypeScript and TSX diagnostics. It does not hide
errors, parse generated JavaScript, or implement a second TSX analyzer. Every
framework-specific result above comes from the application's installed compiler.
