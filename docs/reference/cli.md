# CLI reference

The application CLI is supplied by `@presolve/cli` and exposes the `presolve`
command. In an application created by `pnpm create presolve`, use it through
package scripts:

| Command | Purpose |
| --- | --- |
| `pnpm dev` | Develop the conventional application. |
| `pnpm check` | Validate compiler-admitted application semantics. |
| `pnpm build` | Publish the production artifact set into `dist/`. |
| `pnpm deploy:prepare` | Create and validate a Cloudflare deployment plan without upload. |
| `pnpm deploy` | Validate and deploy the prepared static artifact inventory. |
| `presolve deploy node --prepare` | Build the digest-verified Node host for canonical route loaders and Form server actions without starting it. |
| `presolve explain <source>` | Project compiler-derived facts for explanation and debugging. |
| `presolve help` | Show the commands installed by this release. |
| `presolve version` | Show the installed release version. |

The compiler also has explicit workspace and inspection entry points for
hermetic integration. Those are integration APIs, not required project setup;
use conventional discovery unless you are building compiler tooling.

Always invoke a project-local CLI through pnpm. This avoids a global binary
using a different compiler/framework release than the application's packages.
