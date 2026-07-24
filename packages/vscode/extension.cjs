/*
 * The extension intentionally delegates TypeScript and JSX syntax checking to
 * the workspace TypeScript version. Presolve's compiler diagnostics are only
 * published from compiler products; this entry point must not invent a second
 * source analyzer or suppress TypeScript errors.
 */
function activate(context) {
  const vscode = require("vscode");
  const enabled = () => vscode.workspace
    .getConfiguration("presolve")
    .get("enable", true);

  const disposable = vscode.commands.registerCommand("presolve.status", () => {
    const message = enabled()
      ? "Presolve is using this workspace's TypeScript project configuration."
      : "Presolve workspace integration is disabled.";
    return vscode.window.showInformationMessage(message);
  });
  context.subscriptions.push(disposable);

  return Object.freeze({ enabled });
}

function deactivate() {}

module.exports = { activate, deactivate };
