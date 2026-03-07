  function readState(key, fallback) {
    if (!key) {
      return fallback;
    }
    return key in stateStore ? stateStore[key] : fallback;
  }

  function writeState(key, value) {
    if (!key) {
      return;
    }
    stateStore[key] = value;
    if (canPatchForStateKey(key, value) && updateForBindingsForKey(key)) {
      return;
    }
    scheduleRender();
  }

  function reportRuntimeError(kind, error, context) {
    const details = {
      kind: String(kind || "runtime"),
      message:
        error && typeof error === "object" && typeof error.message === "string"
          ? error.message
          : String(error),
      stack:
        error && typeof error === "object" && typeof error.stack === "string"
          ? error.stack
          : "",
      context: context && typeof context === "object" ? { ...context } : {},
      ts: Date.now(),
    };

    if (typeof window !== "undefined") {
      if (!Array.isArray(window.formoRuntimeErrors)) {
        window.formoRuntimeErrors = [];
      }
      window.formoRuntimeErrors.push(details);

      if (typeof window.formoOnError === "function") {
        try {
          window.formoOnError(details);
        } catch (hookErr) {
          console.error("[formo] error hook failed", hookErr);
        }
      }
    }

    console.error("[formo] runtime error", details);
  }

  function runWithEventBoundary(label, node, handler) {
    try {
      return handler();
    } catch (err) {
      reportRuntimeError("event", err, {
        label: String(label || "event"),
        nodeId: node && node.id ? node.id : "",
        nodeName: node && node.name ? node.name : "",
      });
      return undefined;
    }
  }

  function dispatchAction(name, payload, node, scope) {
    const actionName = String(name || "").trim();
    if (!actionName) {
      return;
    }

    const action = actionHandlers[actionName];
    const event = {
      name: actionName,
      payload,
      nodeId: node.id,
      nodeName: node.name,
      scope: { ...(scope || {}) },
      state: { ...stateStore },
      setState(key, value) {
        writeState(key, value);
      }
    };

    if (typeof action === "undefined") {
      console.info(`[formo] action '${actionName}'`, event);
      return;
    }

    if (typeof action !== "function") {
      reportRuntimeError(
        "action",
        new Error(`action '${actionName}' is not a function`),
        {
          actionName,
          nodeId: node && node.id ? node.id : "",
          nodeName: node && node.name ? node.name : "",
        }
      );
      return;
    }

    try {
      const result = action(event);
      if (result && typeof result.then === "function" && typeof result.catch === "function") {
        result.catch((err) => {
          reportRuntimeError("action", err, {
            actionName,
            nodeId: node && node.id ? node.id : "",
            nodeName: node && node.name ? node.name : "",
          });
        });
      }
    } catch (err) {
      reportRuntimeError("action", err, {
        actionName,
        nodeId: node && node.id ? node.id : "",
        nodeName: node && node.name ? node.name : "",
      });
    }
  }
