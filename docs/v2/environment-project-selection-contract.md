# Project environment-manifest selection contract

Ergonomic V2 commands never discover `.env` files. A project that contains an
authority-proven `environment.public` call must name a previously generated
manifest explicitly:

```sh
presolve environment --file .env.production > environment.manifest.json
presolve check --environment-manifest environment.manifest.json
presolve build --environment-manifest environment.manifest.json
```

`check` and `build` decode the immutable schema-v1 JSON through the compiler
codec. They run TypeScript authority for every decorator-free component module
and every plain module that imports the environment intrinsic. Every proven
read joins to that manifest; any dynamic, server-owned, unprefixed, undeclared,
or absent-manifest read fails the command before publication.

On success, the compiler inserts the validated, source-proven browser artifact
at `environment.browser.json` in the file-route inventory. This selection
contract does not authorize a runtime fallback to process state or dotenv files.
