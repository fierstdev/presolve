#!/usr/bin/env node
import { analyzeV2Authoring } from "../src/index.js";

try {
  let input = "";
  for await (const chunk of process.stdin) input += chunk;
  const request = JSON.parse(input);
  const response = await analyzeV2Authoring(request);
  process.stdout.write(`${JSON.stringify(response)}\n`);
} catch (error) {
  process.stderr.write(`presolve-typescript-authority: ${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
}
