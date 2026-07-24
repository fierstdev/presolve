#!/usr/bin/env node
import { mkdir, writeFile, access } from "node:fs/promises";
import { constants } from "node:fs";
import { basename, resolve } from "node:path";

const targetArgument = process.argv[2];
if (!targetArgument || targetArgument.startsWith("-")) {
  console.error("Usage: npm create presolve <project-directory>");
  process.exit(2);
}

const root = resolve(targetArgument);
try {
  await access(root, constants.F_OK);
  console.error(`Refusing to overwrite existing path: ${root}`);
  process.exit(2);
} catch (error) {
  if (error?.code !== "ENOENT") throw error;
}

const name = packageName(basename(root));
await mkdir(root, { recursive: true });
for (const [relativePath, contents] of Object.entries(template(name))) {
  const path = resolve(root, relativePath);
  await mkdir(resolve(path, ".."), { recursive: true });
  await writeFile(path, contents, "utf8");
}
console.log(`Created ${name}.`);
console.log("Next: install dependencies, then run `npm run dev`.");

function packageName(value) {
  const normalized = value
    .toLowerCase()
    .replace(/[^a-z0-9-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return normalized || "presolve-app";
}

function template(name) {
  return {
    "package.json": `${JSON.stringify({
      name,
      private: true,
      type: "module",
      scripts: {
        dev: "presolve dev",
        build: "presolve build",
        check: "presolve check",
        deploy: "presolve deploy cloudflare",
        "deploy:prepare": "presolve deploy cloudflare --prepare",
      },
      dependencies: { presolve: "0.1.0-alpha" },
      devDependencies: { wrangler: "^4.0.0" },
    }, null, 2)}\n`,
    ".gitignore": "node_modules/\ndist/\n.presolve/\n.dev.vars\n.env*\n",
    "README.md": `# ${name}\n\nA Presolve application.\n\n- \`npm run dev\` builds and serves compiler-published routes.\n- \`npm run build\` emits immutable artifacts in \`dist/\`.\n- \`npm run deploy:prepare\` validates a Cloudflare Workers Static Assets deployment without uploading.\n- \`npm run deploy\` deploys the prepared compiler artifact inventory through Wrangler.\n`,
    "app/routes/index.tsx": `import { component, Component } from "presolve";\n\n@component()\nexport class Home extends Component {\n  render() {\n    return <main><h1>${name}</h1><p>Built by the Presolve compiler.</p><a href="/docs/">Read the docs</a></main>;\n  }\n}\n`,
    "app/routes/docs/index.tsx": `import { component, Component } from "presolve";\n\n@component()\nexport class Docs extends Component {\n  render() {\n    return <main><h1>Presolve documentation</h1><p>Start with compiler-owned components, routes, and deployment.</p><a href="/docs/getting-started/">Getting started</a></main>;\n  }\n}\n`,
    "app/routes/docs/getting-started.tsx": `import { component, Component } from "presolve";\n\n@component()\nexport class GettingStarted extends Component {\n  render() {\n    return <main><h1>Getting started</h1><p>Author components. Presolve publishes static HTML and the minimal runtime artifacts required by your application.</p></main>;\n  }\n}\n`,
  };
}
