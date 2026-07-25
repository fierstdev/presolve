import * as Library from "@compat/library";
import {
  overload,
  shared,
  type Box,
  type ImportedMergeable,
} from "@compat/library";
import { type SubpathToken } from "@compat/library/subpath";
import { aliasedValue, type LocalAlias } from "@fixture/aliases";
import { packageValue } from "#compat-internal";

export { overload as exportedOverload } from "@compat/library";

declare global {
  namespace JSX {
    interface IntrinsicElements {
      "compat-card": { title: string; count?: number };
    }
  }
}

interface Mergeable {
  readonly id: string;
}

interface Mergeable {
  readonly enabled: boolean;
}

const merged: Mergeable = { id: "merged", enabled: true };
const importedMergeable: ImportedMergeable = { id: "imported" };
const box: Box<LocalAlias> = { value: aliasedValue };
const namespaceBox = Library.box("namespace");
const token: SubpathToken = { source: "exports" };
const contextual: Array<(value: string) => number> = [value => value.length];
const element = <compat-card title={box.value.label} count={contextual[0]!("tsx")} />;

class Base {
  protected format(value: string): string {
    return value.toUpperCase();
  }
}

class Derived extends Base {
  #count = 1;

  override format(value: string): string {
    return `${super.format(value)}:${this.#count}`;
  }
}

async function flow(): Promise<string> {
  const selected = await Promise.resolve(overload("typed"));
  if (selected > 0) return new Derived().format(shared.name);
  return packageValue;
}

void flow();
void namespaceBox;
void token;
void merged;
void importedMergeable;
void element;
