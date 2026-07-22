#!/usr/bin/env bash
set -euo pipefail

contract=docs/launch-content-contract.md
site_root=site
content_root=$site_root/content
pages=(index docs architecture examples benchmarks roadmap playground)
linked_files=(
  docs/README.md
  docs/cli-reference.md
  docs/frozen-contract-map.md
  docs/compiler-platform-contract.md
  docs/runtime-contract.md
  docs/production-optimization-contract.md
  docs/resumability-contract.md
  docs/examples-contract.md
  docs/reproducibility-lanes.md
  fixtures/phase-k-benchmarks/README.md
  docs/specifications/phase-l/PHASE_L_REVISED_ROADMAP.md
  docs/specifications/phase-l/PHASE_L_L13_L21_CONTINUATION_CONTRACT.md
  examples/counter/presolve.json
  examples/components-context-slots/presolve.json
  examples/forms/presolve.json
  examples/explicit-workspace/presolve.json
  examples/production-resume/presolve.json
)

test -s "$contract"
test -s "$site_root/README.md"
rg --quiet 'Deployment remains external' "$site_root/README.md"
rg --quiet 'non-functional playground placeholder' "$site_root/README.md"
rg --quiet 'repository-local launch content only' "$contract"
rg --quiet 'claim comparative numbers' "$contract"

for page in "${pages[@]}"; do
  file="$content_root/$page.md"
  test -s "$file"
  rg --quiet '^schema: presolve.launch-content$' "$file"
  rg --quiet '^version: 1$' "$file"
  rg --quiet '^presolve_version: 0.1.0-alpha$' "$file"
done

for file in "${linked_files[@]}"; do
  test -s "$file"
done

node --input-type=module - "$content_root" <<'NODE'
import { readdir, readFile, stat } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

const contentRoot = process.argv[2];
const allowedExternal = 'https://github.com/fierstdev/presolve';
const pages = (await readdir(contentRoot)).filter((entry) => entry.endsWith('.md'));

for (const page of pages) {
  const source = resolve(contentRoot, page);
  const content = await readFile(source, 'utf8');
  for (const match of content.matchAll(/\[[^\]]*\]\(([^)\s]+)\)/g)) {
    const target = match[1];
    if (target.startsWith('https://')) {
      if (target !== allowedExternal) {
        throw new Error(`${page}: unexpected external link ${target}`);
      }
      continue;
    }
    if (target.startsWith('#')) continue;
    try {
      await stat(resolve(dirname(source), target));
    } catch {
      throw new Error(`${page}: missing local link ${target}`);
    }
  }
}
NODE

rg --quiet 'github.com/fierstdev/presolve' $content_root/index.md
rg --quiet 'exactly the examples defined' $content_root/examples.md
rg --quiet 'non-gating' $content_root/benchmarks.md
rg --quiet 'Non-functional placeholder' $content_root/playground.md
rg --quiet 'no editor, compiler endpoint, source upload, source' $content_root/playground.md
rg --quiet '../../docs/README.md' "$content_root/docs.md"
rg --quiet '../../docs/compiler-platform-contract.md' "$content_root/architecture.md"
rg --quiet '../../examples/counter/presolve.json' "$content_root/examples.md"
rg --quiet '../../fixtures/phase-k-benchmarks/README.md' "$content_root/benchmarks.md"
rg --quiet '../../docs/specifications/phase-l/PHASE_L_REVISED_ROADMAP.md' "$content_root/roadmap.md"

git diff --check
