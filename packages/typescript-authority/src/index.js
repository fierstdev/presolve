import { API, SymbolFlags } from "@typescript/native/unstable/async";
import { getTokenAtPosition } from "@typescript/native/unstable/ast";
import { dirname, relative, resolve, sep } from "node:path";

export const TYPESCRIPT_SEMANTIC_AUTHORITY_SCHEMA_VERSION = 1;
export const PRIMARY_TYPESCRIPT_VERSION = "7.0.2";

/**
 * Queries TypeScript's semantic engine and returns only Presolve-owned,
 * serializable products. Compiler domain modules must consume this boundary
 * rather than importing concrete TypeScript APIs.
 */
export async function analyzeTypeScriptProject(request) {
  validateRequest(request);
  const configFile = resolve(request.cwd ?? process.cwd(), request.configFile);
  const api = new API({ cwd: request.cwd ?? process.cwd() });

  try {
    const config = await api.parseConfigFile(configFile);
    const snapshot = await api.updateSnapshot({ openProjects: [configFile] });
    const project = snapshot.getProject(configFile);
    if (!project) throw new Error(`TypeScript did not open project ${configFile}`);

    const queries = request.queries ?? {};
    const diagnostics = await collectDiagnostics(project.program, config);
    const response = {
      schemaVersion: TYPESCRIPT_SEMANTIC_AUTHORITY_SCHEMA_VERSION,
      typeScriptVersion: PRIMARY_TYPESCRIPT_VERSION,
      project: {
        configFile: project.configFileName,
        rootFiles: [...project.rootFiles].sort(),
      },
      diagnostics,
      symbols: await querySymbols(project, queries.symbols ?? []),
      types: await queryTypes(project, queries.types ?? []),
      contextualTypes: await queryContextualTypes(project, queries.contextualTypes ?? []),
      signatures: await querySignatures(project, queries.signatures ?? []),
      assignability: await queryAssignability(project, queries.assignability ?? []),
      modules: await queryModules(project, queries.modules ?? []),
    };
    await snapshot.dispose();
    return response;
  } finally {
    await api.close();
  }
}

async function collectDiagnostics(program, config) {
  const sources = [
    ["config", config.diagnostics ?? []],
    ["program", await program.getProgramDiagnostics()],
    ["syntactic", await program.getSyntacticDiagnostics()],
    ["bind", await program.getBindDiagnostics()],
    ["semantic", await program.getSemanticDiagnostics()],
  ];
  return sources
    .flatMap(([source, diagnostics]) => diagnostics.map(diagnostic => serializeDiagnostic(source, diagnostic)))
    .sort(compareDiagnostics);
}

async function querySymbols(project, queries) {
  return Promise.all(queries.map(async query => {
    const symbol = await symbolAt(project, query);
    return { id: query.id, symbol: await serializeSymbol(project, symbol) };
  }));
}

async function queryTypes(project, queries) {
  return Promise.all(queries.map(async query => {
    const type = await typeAt(project, query);
    return { id: query.id, type: await serializeType(project.checker, type) };
  }));
}

async function queryContextualTypes(project, queries) {
  return Promise.all(queries.map(async query => {
    const { file, token } = await tokenAt(project, query);
    const type = await nearestContextualType(project.checker, token, file);
    return { id: query.id, type: await serializeType(project.checker, type) };
  }));
}

async function querySignatures(project, queries) {
  return Promise.all(queries.map(async query => {
    const { file, token } = await tokenAt(project, query);
    const signature = await nearestSignature(project.checker, token);
    return { id: query.id, signature: await serializeSignature(project.checker, signature) };
  }));
}

async function queryAssignability(project, queries) {
  return Promise.all(queries.map(async query => {
    const source = await typeAt(project, query.source);
    const target = await typeAt(project, query.target);
    return {
      id: query.id,
      assignable: Boolean(source && target && await project.checker.isTypeAssignableTo(source, target)),
      source: await serializeType(project.checker, source),
      target: await serializeType(project.checker, target),
    };
  }));
}

async function queryModules(project, queries) {
  return Promise.all(queries.map(async query => {
    const { file, token } = await tokenAt(project, query);
    const symbol = await project.checker.getSymbolAtPosition(file.fileName, query.position);
    const module = await serializeSymbol(project, symbol);
    return {
      id: query.id,
      module: module && {
        ...module,
        specifier: token.getText(file).replace(/^(?:"|')|(?:"|')$/g, ""),
        resolvedModulePaths: module.identity.declarationModules,
      },
    };
  }));
}

async function symbolAt(project, query) {
  const { file } = await tokenAt(project, query);
  return project.checker.getSymbolAtPosition(file.fileName, query.position);
}

async function typeAt(project, query) {
  const { file } = await tokenAt(project, query);
  return project.checker.getTypeAtPosition(file.fileName, query.position);
}

async function tokenAt(project, query) {
  validatePositionQuery(query);
  const fileName = resolve(query.file);
  const file = await project.program.getSourceFile(fileName);
  if (!file) throw new Error(`TypeScript project does not contain ${fileName}`);
  if (query.position < 0 || query.position > file.text.length) {
    throw new RangeError(`position ${query.position} is outside ${fileName}`);
  }
  return { file, token: getTokenAtPosition(file, query.position) };
}

async function nearestContextualType(checker, token, file) {
  for (let node = token; node; node = node.parent) {
    const type = await checker.getContextualType(node);
    if (type) return type;
    if (node === file) break;
  }
  return undefined;
}

async function nearestSignature(checker, token) {
  for (let node = token; node; node = node.parent) {
    try {
      const signature = await checker.getResolvedSignature(node);
      if (signature) return signature;
    } catch {
      // TypeScript only resolves signatures for invocation-like nodes.
    }
  }
  return undefined;
}

async function serializeSymbol(project, symbol) {
  if (!symbol) return undefined;
  const { checker } = project;
  const declarationPaths = symbol.declarations
    .map(declaration => String(declaration.path))
    .sort();
  const serialized = {
    name: symbol.name,
    flags: symbol.flags,
    declarationPaths,
    identity: identityForSymbol(project, symbol, declarationPaths),
  };
  if ((symbol.flags & SymbolFlags.Alias) === 0) return serialized;

  const target = await checker.getAliasedSymbol(symbol);
  return {
    ...serialized,
    aliasTarget: {
      name: target.name,
      flags: target.flags,
      declarationPaths: target.declarations.map(declaration => String(declaration.path)).sort(),
      identity: identityForSymbol(
        project,
        target,
        target.declarations.map(declaration => String(declaration.path)).sort(),
      ),
      unknown: await checker.isUnknownSymbol(target),
    },
  };
}

function identityForSymbol(project, symbol, declarationPaths) {
  const projectRoot = dirname(String(project.id));
  return {
    name: symbol.name,
    flags: symbol.flags,
    declarationModules: [...new Set(declarationPaths.map(path => normalizeProjectPath(projectRoot, path)))],
  };
}

function normalizeProjectPath(projectRoot, path) {
  const relativePath = relative(projectRoot, path);
  return relativePath.split(sep).join("/") || ".";
}

async function serializeType(checker, type) {
  if (!type) return undefined;
  return {
    text: await checker.typeToString(type),
    flags: type.flags,
    error: type.isErrorType(),
  };
}

async function serializeSignature(checker, signature) {
  if (!signature) return undefined;
  const parameters = await signature.getParameters();
  return {
    parameterTypes: await Promise.all(parameters.map(async parameter => ({
      name: parameter.name,
      type: await serializeType(checker, await checker.getTypeOfSymbol(parameter)),
    }))),
    returnType: await serializeType(checker, await checker.getReturnTypeOfSignature(signature)),
  };
}

function serializeDiagnostic(source, diagnostic) {
  return {
    source,
    code: diagnostic.code,
    category: diagnostic.category,
    file: diagnostic.fileName,
    start: diagnostic.pos,
    end: diagnostic.end,
    message: diagnostic.text,
  };
}

function compareDiagnostics(left, right) {
  return (left.file ?? "").localeCompare(right.file ?? "")
    || left.start - right.start
    || left.code - right.code
    || left.source.localeCompare(right.source);
}

function validateRequest(request) {
  if (!request || typeof request !== "object" || typeof request.configFile !== "string") {
    throw new TypeError("TypeScript authority requests require a configFile");
  }
  if (request.cwd !== undefined && typeof request.cwd !== "string") {
    throw new TypeError("TypeScript authority cwd must be a string");
  }
  if (request.queries !== undefined && (typeof request.queries !== "object" || Array.isArray(request.queries))) {
    throw new TypeError("TypeScript authority queries must be an object");
  }
}

function validatePositionQuery(query) {
  if (!query || typeof query.file !== "string" || !Number.isInteger(query.position)) {
    throw new TypeError("TypeScript authority position queries require file and integer position");
  }
}
