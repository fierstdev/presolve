#!/usr/bin/env node
import { mkdir, writeFile, access, readFile } from "node:fs/promises";
import { constants } from "node:fs";
import { basename, resolve } from "node:path";
import { stdin, stdout } from "node:process";
import { createInterface } from "node:readline/promises";

const packageManifest = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
);
const argumentsList = process.argv.slice(2);

if (argumentsList.includes("--help") || argumentsList.includes("-h")) {
  printHelp();
  process.exit(0);
}
if (argumentsList.includes("--version") || argumentsList.includes("-v")) {
  console.log(packageManifest.version);
  process.exit(0);
}
const unknownOption = argumentsList.find(argument => argument.startsWith("-"));
if (unknownOption) {
  console.error(`Unknown option: ${unknownOption}`);
  console.error("Run `pnpm create presolve --help` for usage.");
  process.exit(2);
}
if (argumentsList.length > 1) {
  console.error("create-presolve accepts one project directory.");
  process.exit(2);
}

let targetArgument = argumentsList[0];
if (!targetArgument) {
  if (!stdin.isTTY || !stdout.isTTY) {
    console.error("Usage: pnpm create presolve <project-directory>");
    process.exit(2);
  }
  const prompt = createInterface({ input: stdin, output: stdout });
  try {
    targetArgument = (await prompt.question(
      "Where should Presolve create your application? (presolve-app) ",
    )).trim() || "presolve-app";
  } finally {
    prompt.close();
  }
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
console.log("");
console.log("Next:");
console.log(`  cd ${targetArgument}`);
console.log("  pnpm install");
console.log("  pnpm dev");

function packageName(value) {
  const normalized = value
    .toLowerCase()
    .replace(/[^a-z0-9-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return normalized || "presolve-app";
}

function printHelp() {
  console.log(`Create a Presolve application.

Usage:
  pnpm create presolve <project-directory>
  pnpm create presolve

Options:
  -h, --help       Show this help
  -v, --version    Show the create-presolve version

The creator never overwrites an existing path. When a terminal is available,
omitting the directory starts a prompt.`);
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
      dependencies: { presolve: "npm:@presolve/framework@0.2.0-beta.27" },
      devDependencies: {
        "@presolve/cli": "0.2.0-beta.27",
        "@presolve/typescript-authority": "0.2.0-beta.27",
        "typescript": "npm:typescript@^7.0.2",
        "vite": "^7.0.0",
        "wrangler": "^4.0.0",
      },
    }, null, 2)}\n`,
    "pnpm-workspace.yaml": `# Vite and Wrangler require these audited installers under pnpm 11.
allowBuilds:
  esbuild: true
  workerd: true
`,
    ".gitignore": "node_modules/\ndist/\n.presolve/\n.dev.vars\n.env*\n!.env.example\n",
    "README.md": `# ${name}\n\nA compiler-founded Presolve application with complete static HTML and exact browser behavior.\n\n## Start\n\n\`\`\`sh\npnpm install\npnpm dev\n\`\`\`\n\n- \`pnpm check\` validates TypeScript and compiler-owned application semantics.\n- \`pnpm build\` emits the complete immutable publication in \`dist/\`.\n- \`pnpm deploy:prepare\` validates a Cloudflare Workers Static Assets release without uploading.\n- \`pnpm deploy\` deploys the compiler-issued inventory through Wrangler.\n- \`pnpm deploy:node:prepare\` writes a Node release inventory and static host under \`.presolve/node/\`. Routes with loaders or server actions remain explicitly Node-required until their capability-specific executor is available.\n\n## Why the project is structured this way\n\nPresolve separates authored responsibilities so document metadata, application composition, route landmarks, presentation, and compiler output never compete for ownership:\n\n- \`app/index.html\` is the document frame. It owns stable metadata and must contain exactly one \`{{ head }}\`, \`{{ app }}\`, and \`{{ runtime }}\` placeholder. Presolve fills those placeholders for every route.\n- \`app/app.tsx\` is the application shell. Shared navigation, providers, theme UI, and the route \`<slot />\` belong here. It deliberately does not render \`<html>\`, \`<head>\`, or a page-level \`<main>\`.\n- \`app/routes/\` is the route graph. Each route owns its page content and primary landmark; no parallel router table is required.\n- \`app/app.css\` is the global stylesheet. Presolve copies its exact bytes to \`/app.css\`, emits an immutable \`/app.<sha256>.css\`, and links the immutable file from the generated document head.\n- \`app/components/\` holds reusable application components without creating routes.\n- \`public/\` holds files served from root URLs, such as \`/favicon.svg\`. Presolve copies and inventories them during build.\n- \`assets/\` is reserved for CSS, media, and other inputs owned by an explicit Vite adapter integration. Placing a file there does not create framework semantics by itself.\n- \`server/\` is server-owned source and \`tests/\` is application verification source. Neither becomes browser code just because it exists.\n- \`dist/\` and \`.presolve/\` are generated. Do not edit or commit them.\n\nThe compatibility inputs \`app/layout.tsx\` and \`styles/\` remain readable for older beta projects, but new source should use the canonical files above.\n\n## Styling and Vite\n\nThe default styling path is intentionally direct: components and routes render ordinary \`class\` attributes, selectors live in \`app/app.css\`, and the browser connects them through the normal global cascade. Presolve publishes those exact bytes, adds the document-level stylesheet link to every route head, and hot-swaps rebuilt CSS during \`pnpm dev\` without replacing component state, focus, or scroll position. Responsive media queries, custom properties, selectors, animations, and modern browser CSS work without a client styling runtime.\n\nVite has a separate, bounded role. The project-local \`vite\` installation bundles compiler-authorized package Actions, Standard Schema validators, and form-submission capabilities. The public \`@presolve/vite\` adapter can also process caller-declared CSS Modules, PostCSS/Tailwind entries, imported fonts/media, source maps, and public directories for custom integrations. Vite never defines routes, component identity, state, resumability, or deployment eligibility.\n\nFor Tailwind, compile a Tailwind input into \`app/app.css\` before \`presolve dev\` or \`presolve build\`, and keep the Tailwind watcher writing that file while developing. The browser still receives ordinary content-addressed CSS and no Tailwind runtime.\n\n\`pnpm dev\` watches authored inputs and republishes from the compiler. CSS edits hot-swap the stylesheet; semantic edits use a safe full reload unless a narrower compiler HMR product proves state compatibility. If an edit fails to compile, the last good page stays available and the browser shows the compiler diagnostic until the project recovers.\n\nOnly explicit \`PRESOLVE_PUBLIC_*\` environment values are browser-eligible. Open the whole project in VS Code and install the recommended **Presolve** extension so TypeScript and compiler diagnostics use the same release train as the build.\n`,
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
    "app/components/README.md": "Shared application components belong here. Import them from routes or other components; only files under app/routes declare URLs.\n",
    "server/README.md": "Server-owned source belongs here. Do not import server values into browser-owned paths.\n",
    "app/app.css": `:root {
  color-scheme: dark;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #07090f;
  color: #f7f8fc;
}

* { box-sizing: border-box; }
html { background: #07090f; -webkit-text-size-adjust: 100%; text-size-adjust: 100%; }
body { min-width: 320px; margin: 0; background: radial-gradient(circle at 15% 0%, #241c45 0, transparent 28rem), #07090f; }
a { color: #8fe6fa; }
button, input, textarea, select { font: inherit; }
:focus-visible { outline: 3px solid #75dcf5; outline-offset: 3px; }

.skip-link { position: fixed; top: -5rem; left: 1rem; z-index: 10; padding: .75rem 1rem; background: #fff; color: #07090f; }
.skip-link:focus { top: 1rem; }
.app-header, .app-footer, .page { width: min(100% - 2rem, 68rem); margin-inline: auto; }
.app-header { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding-block: 1rem; border-bottom: 1px solid #ffffff1f; }
.brand { color: #fff; font-weight: 800; text-decoration: none; }
.app-header nav { display: flex; flex-wrap: wrap; gap: 1rem; }
.page { padding-block: clamp(3rem, 10vw, 7rem); }
.eyebrow { margin: 0; color: #9d8bff; font-size: .75rem; font-weight: 800; letter-spacing: .12em; text-transform: uppercase; }
h1 { max-width: 12ch; margin: .8rem 0 0; font-size: clamp(2.75rem, 12vw, 6rem); letter-spacing: -.065em; line-height: .95; }
.lede { max-width: 42rem; margin-top: 1.25rem; color: #b5bfd0; font-size: clamp(1rem, 3vw, 1.2rem); line-height: 1.7; }
.counter { display: grid; gap: 1rem; max-width: 30rem; margin-top: 2rem; padding: 1.25rem; border: 1px solid #9d8bff55; border-radius: 1rem; background: #ffffff0a; }
.counter output { font-size: 1.5rem; font-weight: 800; }
.counter button { width: fit-content; min-height: 2.75rem; padding: .65rem 1rem; border: 0; border-radius: .65rem; background: linear-gradient(120deg, #ad9eff, #75dcf5); color: #07090f; font-weight: 800; cursor: pointer; }
.app-footer { padding-block: 1.5rem 2.5rem; border-top: 1px solid #ffffff1f; color: #8691a4; font-size: .85rem; }

@media (min-width: 48rem) {
  .app-header, .app-footer, .page { width: min(100% - 4rem, 68rem); }
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { scroll-behavior: auto !important; transition-duration: .01ms !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; }
}
`,
    "app/index.html": `<!doctype html>
<html lang="en">
<head>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="${name}, built with Presolve.">
  <meta name="theme-color" content="#07090f">
  <link rel="icon" href="/favicon.svg" type="image/svg+xml">
{{ head }}
</head>
<body>
{{ app }}{{ runtime }}
</body>
</html>
`,
    "assets/README.md": "Explicit @presolve/vite integration inputs such as imported CSS, fonts, and media belong here. This directory is not automatically copied and creates no Presolve semantic identity.\n",
    "public/favicon.svg": `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 60 72" role="img" aria-label="Presolve">
  <rect width="60" height="72" rx="10" fill="#07090f"/>
  <path fill="#f7f8fc" d="M8 8h44v14H22v42H8z"/>
  <path fill="#f59e0b" d="M26 26h26v30H26z"/>
</svg>
`,
    "public/robots.txt": "User-agent: *\nAllow: /\n",
    "tests/README.md": "Application tests belong here; Vitest and Playwright adapters consume published products.\n",
    "app/app.tsx": `import { Component, slot, type SlotContent } from "presolve";

export class App extends Component {
  children: SlotContent = slot();

  render() {
    return (
      <div class="app-shell">
        <a class="skip-link" href="#content">Skip to content</a>
        <header class="app-header">
          <a class="brand" href="/">${name}</a>
          <nav aria-label="Primary">
            <a href="/">Home</a>
            <a href="/docs/">Docs</a>
          </nav>
        </header>
        <slot />
        <footer class="app-footer">Built from compiler-owned application products.</footer>
      </div>
    );
  }
}
`,
    "app/routes/index.tsx": `import { action, Component, state } from "presolve";

export class Home extends Component {
  count = state(0);

  get nextCount() {
    return this.count + 1;
  }

  increment = action(() => {
    this.count += 1;
  });

  render() {
    return (
      <main class="page" id="content">
        <p class="eyebrow">Presolve starter</p>
        <h1>Ship the page. Resume the behavior.</h1>
        <p class="lede">This route begins as complete HTML. The counter resumes from compiler-issued state and event artifacts without hydrating a component tree.</p>
        <section class="counter" aria-labelledby="counter-title">
          <h2 id="counter-title">Interactive proof</h2>
          <output aria-live="polite">Count: {this.count}</output>
          <p>Next increment: <strong>{this.nextCount}</strong></p>
          <button type="button" onClick={this.increment}>Increment</button>
        </section>
      </main>
    );
  }
}
`,
    "app/routes/docs/index.tsx": `import { Component } from "presolve";\n\nexport class Docs extends Component {\n  render() {\n    return <main><h1>Presolve documentation</h1><p>Start with compiler-owned components, routes, and deployment.</p><a href="/docs/getting-started/">Getting started</a></main>;\n  }\n}\n`,
    "app/routes/docs/getting-started.tsx": `import { Component } from "presolve";\n\nexport class GettingStarted extends Component {\n  render() {\n    return <main><h1>Getting started</h1><p>Author components. Presolve publishes static HTML and the minimal runtime artifacts required by your application.</p></main>;\n  }\n}\n`,
  };
}
