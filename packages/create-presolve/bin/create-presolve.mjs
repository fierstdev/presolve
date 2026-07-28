#!/usr/bin/env node
import { mkdir, writeFile, access } from "node:fs/promises";
import { constants } from "node:fs";
import { basename, resolve } from "node:path";

const targetArgument = process.argv[2];
if (!targetArgument || targetArgument.startsWith("-")) {
  console.error("Usage: pnpm create presolve <project-directory>");
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
console.log("Next: `cd " + name + " && pnpm install && pnpm dev`.");

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
      packageManager: "pnpm@11.17.0",
      scripts: {
        dev: "presolve dev",
        build: "presolve build",
        check: "presolve check",
        deploy: "presolve deploy cloudflare",
        "deploy:prepare": "presolve deploy cloudflare --prepare",
        "deploy:node:prepare": "presolve deploy node --prepare",
      },
      dependencies: { presolve: "npm:@presolve/framework@0.2.0-beta.5" },
      devDependencies: {
        "@presolve/cli": "0.2.0-beta.5",
        "@presolve/typescript-authority": "0.2.0-beta.5",
        "typescript": "npm:typescript@^7.0.2",
        "wrangler": "^4.0.0",
      },
    }, null, 2)}\n`,
    ".gitignore": "node_modules/\ndist/\n.presolve/\n.dev.vars\n.env*\n",
    "README.md": `# ${name}\n\nA Presolve application.\n\n## Development\n\n\`pnpm dev\` builds and serves compiler-published routes.\n\n\`pnpm build\` emits immutable artifacts in \`dist/\`.\n\n\`pnpm deploy:prepare\` validates a Cloudflare Workers Static Assets deployment without uploading.\n\n\`pnpm deploy\` deploys the prepared compiler artifact inventory through Wrangler.\n\n\`pnpm deploy:node:prepare\` writes a Node release inventory and static host under \`.presolve/node/\`. Routes with loaders or server actions remain explicitly Node-required until their capability-specific executor is available.\n\n## Project layout\n\n\`app/app.tsx\` is the application shell: shared providers, navigation, and footer belong there. \`app/app.css\` is the global stylesheet; Presolve publishes it as \`/app.css\` and includes it in the document head automatically. \`app/index.html\` is a compiler template rather than a traditional entry point: it must contain exactly one \`{{ head }}\`, \`{{ app }}\`, and \`{{ runtime }}\` placeholder. Routes live in \`app/routes\`, shared application source belongs in \`app/components\`, server-owned source in \`server\`, and application tests in \`tests\`. Files in \`public/\` are copied to the root of \`dist/\` and included in the deployment inventory; use \`assets/\` for adapter-owned imported assets and transforms. \`app/layout.tsx\` and \`styles/\` remain supported as beta compatibility paths, but new applications should use the canonical \`app/\` files. Only \`PRESOLVE_PUBLIC_*\` values are browser-eligible.\n\nVS Code will use the TypeScript configuration in this project. Install the **Presolve** extension for compiler-language tooling during the beta.\n`,
    "tsconfig.json": `${JSON.stringify({
      compilerOptions: {
        target: "ES2022",
        module: "NodeNext",
        moduleResolution: "NodeNext",
        strict: true,
        noEmit: true,
        jsx: "preserve",
        skipLibCheck: true,
      },
      include: ["app/**/*.ts", "app/**/*.tsx", "server/**/*.ts"],
    }, null, 2)}\n`,
    ".vscode/extensions.json": `${JSON.stringify({ recommendations: ["fierstdev.presolve-vscode"] }, null, 2)}\n`,
    ".env.example": "# Browser-visible values must use the PRESOLVE_PUBLIC_ prefix.\nPRESOLVE_PUBLIC_APP_NAME=Presolve App\n",
    "app/components/README.md": "Shared application components belong here. Routes are declared only under app/routes.\n",
    "server/README.md": "Server-owned source belongs here. Do not import server values into browser-owned paths.\n",
    "app/app.css": "/* Global application styles are included from the document head. */\n",
    "app/index.html": "<!doctype html>\n<html lang=\"en\">\n<head>\n{{ head }}\n</head>\n<body>\n{{ app }}{{ runtime }}\n</body>\n</html>\n",
    "assets/README.md": "Imported Vite asset inputs belong here.\n",
    "public/robots.txt": "User-agent: *\nAllow: /\n",
    "tests/README.md": "Application tests belong here; Vitest and Playwright adapters consume published products.\n",
    "app/app.tsx": `import { Component, slot, type SlotContent } from "presolve";\n\nexport class App extends Component {\n  children: SlotContent = slot();\n\n  render() {\n    return <div class="app-shell"><slot /></div>;\n  }\n}\n`,
    "app/routes/index.tsx": `import { Component } from "presolve";\n\nexport class Home extends Component {\n  render() {\n    return <main><h1>${name}</h1><p>Built by the Presolve compiler.</p><a href="/docs/">Read the docs</a></main>;\n  }\n}\n`,
    "app/routes/docs/index.tsx": `import { Component } from "presolve";\n\nexport class Docs extends Component {\n  render() {\n    return <main><h1>Presolve documentation</h1><p>Start with compiler-owned components, routes, and deployment.</p><a href="/docs/getting-started/">Getting started</a></main>;\n  }\n}\n`,
    "app/routes/docs/getting-started.tsx": `import { Component } from "presolve";\n\nexport class GettingStarted extends Component {\n  render() {\n    return <main><h1>Getting started</h1><p>Author components. Presolve publishes static HTML and the minimal runtime artifacts required by your application.</p></main>;\n  }\n}\n`,
  };
}
