#!/usr/bin/env node

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { resolve, dirname } from "node:path";

const root = resolve(import.meta.dirname, "..");
const documents = collect(resolve(root, "docs"));
const publicGuides = collect(resolve(root, "docs", "guide"));

for (const document of documents) {
  const source = readFileSync(document, "utf8");
  if (/\bPhase\s+[A-Z]/.test(source) || /docs\/(archive|specifications)|\bL\d{1,2}\b/.test(source)) {
    throw new Error(`${relative(document)} contains retired planning language`);
  }
  for (const match of source.matchAll(/\]\(([^)#]+)(?:#[^)]+)?\)/g)) {
    const target = match[1];
    if (/^[a-z]+:/i.test(target) || target.startsWith("/")) continue;
    const path = resolve(dirname(document), target);
    if (!existsSync(path)) throw new Error(`${relative(document)} links to missing ${target}`);
  }
}

for (const guide of publicGuides) {
  const source = readFileSync(guide, "utf8");
  if (
    /@(component|action|computed|effect|context|provide|consume|form|field|submit|resource|slot|loader|serverAction|opaque)\b/.test(source) &&
    !/(legacy (compatibility|decorator)|alpha-compatibility|compatibility form)/i.test(source)
  ) {
    throw new Error(
      `${relative(guide)} presents decorator syntax without marking it as alpha compatibility`
    );
  }
}

console.log(`Validated ${documents.length} public documentation files.`);

function collect(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(directory, entry.name);
    return entry.isDirectory() ? collect(path) : entry.name.endsWith(".md") ? [path] : [];
  });
}

function relative(path) {
  return path.slice(root.length + 1);
}
