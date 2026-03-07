  function render() {
    const focusSnap = captureFocusSnapshot();
    forBindingsByStateKey.clear();
    const root = buildNode(entry.rootNodeId, nodes, {});
    mount.replaceChildren(root);
    restoreFocusSnapshot(focusSnap);
  }

  function scheduleRender() {
    if (renderQueued) {
      return;
    }
    renderQueued = true;
    queueMicrotask(() => {
      renderQueued = false;
      render();
    });
  }

  function initializeStateDefaults() {
    for (const node of nodes.values()) {
      switch (node.name) {
        case "Input": {
          const key = propAsString(node, "value");
          if (key && !(key in stateStore)) {
            stateStore[key] = "";
          }
          break;
        }
        case "Checkbox":
        case "Switch": {
          const key = propAsString(node, "checked");
          if (key && !(key in stateStore)) {
            stateStore[key] = false;
          }
          break;
        }
        case "Modal": {
          const key = propAsString(node, "open");
          if (key && !(key in stateStore)) {
            stateStore[key] = false;
          }
          break;
        }
        case "For": {
          const key = propLiteralString(node, "each");
          if (key && isIdentifierRef(key) && !(key in stateStore)) {
            stateStore[key] = [];
          }
          break;
        }
        default:
          break;
      }
    }
  }