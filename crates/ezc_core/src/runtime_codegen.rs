const RUNTIME_STUB: &str = r#"(() => {
  "use strict";

  const MANIFEST_ELEMENT_ID = "ez-template-manifest";

  function readManifest() {
    const element = document.getElementById(MANIFEST_ELEMENT_ID);

    if (!(element instanceof HTMLScriptElement)) {
      throw new Error(
        `Missing template manifest script #${MANIFEST_ELEMENT_ID}`
      );
    }

    return JSON.parse(element.textContent ?? "");
  }

  function collectBindingAnchorIds() {
    const ids = new Set();
    const walker = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_COMMENT
    );

    while (walker.nextNode()) {
      const value = (walker.currentNode.nodeValue ?? "").trim();
      const match = /^ez-binding:([^:]+):/.exec(value);

      if (match !== null) {
        ids.add(match[1]);
      }
    }

    return ids;
  }

  function collectMissingAnchors(manifest) {
    const missing = [];
    const bindingAnchorIds = collectBindingAnchorIds();

    for (const component of manifest.components ?? []) {
      for (const node of component.template?.nodes ?? []) {
        if (node.kind === "element") {
          const selector = `[data-ez-node="${node.id}"]`;

          if (document.querySelector(selector) === null) {
            missing.push({
              id: node.id,
              kind: node.kind
            });
          }
        }

        if (
          node.kind === "binding" &&
          !bindingAnchorIds.has(node.id)
        ) {
          missing.push({
            id: node.id,
            kind: node.kind
          });
        }
      }
    }

    return missing;
  }

  function boot() {
    try {
      const manifest = readManifest();
      const missingAnchors = collectMissingAnchors(manifest);
      const status = missingAnchors.length === 0 ? "ready" : "error";

      const runtimeState = {
        manifest,
        missingAnchors
      };

      document.documentElement.dataset.ezRuntime = status;
      window.__EDGEZERO__ = runtimeState;

      document.dispatchEvent(
        new CustomEvent("edgezero:ready", {
          detail: runtimeState
        })
      );

      if (missingAnchors.length > 0) {
        console.error(
          "[EdgeZero] Missing template anchors",
          missingAnchors
        );
      }
    } catch (error) {
      document.documentElement.dataset.ezRuntime = "error";

      console.error(
        "[EdgeZero] Runtime boot failed",
        error
      );
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, {
      once: true
    });
  } else {
    boot();
  }
})();
"#;

pub fn generate_runtime_stub() -> String {
    RUNTIME_STUB.to_string()
}

#[cfg(test)]
mod tests {
    use super::generate_runtime_stub;

    #[test]
    fn emits_runtime_manifest_bootstrap() {
        let runtime = generate_runtime_stub();

        assert!(runtime.contains("ez-template-manifest"));
        assert!(runtime.contains("data-ez-node"));
        assert!(runtime.contains("ez-binding:"));
        assert!(runtime.contains("dataset.ezRuntime"));
        assert!(runtime.contains("edgezero:ready"));
        assert!(runtime.contains("window.__EDGEZERO__"));
    }
}
