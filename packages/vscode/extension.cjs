const { existsSync } = require("node:fs");
const { join } = require("node:path");

/* The extension makes the generated workspace legible to VS Code. TypeScript
 * still owns TypeScript/TSX diagnostics; Presolve never suppresses them or
 * creates a second source analyzer. */
function activate(context) {
  const vscode = require("vscode");
  const enabled = () => vscode.workspace
    .getConfiguration("presolve")
    .get("enable", true);

  const status = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  status.command = "presolve.status";

  const refresh = () => {
    const workspace = inspectWorkspace(vscode);
    status.text = enabled() ? "$(check) Presolve" : "$(circle-slash) Presolve";
    status.tooltip = workspace.message;
    if (enabled()) status.show(); else status.hide();
    return workspace;
  };

  const disposable = vscode.commands.registerCommand("presolve.status", () => {
    return vscode.window.showInformationMessage(refresh().message);
  });
  const changes = vscode.workspace.onDidChangeWorkspaceFolders(refresh);
  const settings = vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration("presolve.enable")) refresh();
  });
  context.subscriptions.push(disposable, changes, settings, status);
  refresh();

  return Object.freeze({ enabled });
}

function deactivate() {}

function inspectWorkspace(vscode) {
  const root = vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath;
  if (!root) return { configured: false, message: "Open a Presolve project folder to enable workspace integration." };
  if (!existsSync(join(root, "tsconfig.json"))) {
    return { configured: false, message: "This folder has no tsconfig.json. Run pnpm create presolve or add a project TypeScript configuration." };
  }
  return { configured: true, message: "Presolve is using this workspace's TypeScript project configuration." };
}

module.exports = { activate, deactivate, inspectWorkspace };
