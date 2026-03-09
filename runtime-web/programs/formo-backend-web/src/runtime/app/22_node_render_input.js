      case "Input": {
        element = el("input", "fm-input");
        childMount = element;
        const explicitStateKey =
          propAsString(node, "value", scope) ||
          propAsString(node, "bind", scope) ||
          propAsString(node, "name", scope);
        const stateKey = explicitStateKey || `__local_input::${node.id}`;
        const placeholder = propAsString(node, "placeholder", scope);
        const inputType = propAsString(node, "inputType", scope) || "text";
        const actionName = propAsString(node, "onChange", scope);
        const defaultValue = propAsString(node, "defaultValue", scope);
        element.type = inputType;
        element.disabled = propAsBool(node, "disabled", false, scope);
        if (placeholder) {
          element.placeholder = placeholder;
        }

        if (!(stateKey in stateStore)) {
          stateStore[stateKey] = defaultValue || "";
        }
        if (explicitStateKey) {
          element.dataset.stateKey = explicitStateKey;
        }
        element.value = String(readState(stateKey, ""));
        element.addEventListener("input", () => {
          runWithEventBoundary("Input.input", node, () => {
            writeState(stateKey, element.value);
            if (explicitStateKey) {
              dispatchAction(actionName, element.value, node, scope);
            }
          });
        });
        break;
      }
