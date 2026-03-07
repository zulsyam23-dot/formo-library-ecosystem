      case "Input": {
        element = el("input", "fm-input");
        childMount = element;
        const stateKey = propAsString(node, "value", scope);
        const placeholder = propAsString(node, "placeholder", scope);
        const inputType = propAsString(node, "inputType", scope) || "text";
        const actionName = propAsString(node, "onChange", scope);
        element.type = inputType;
        element.disabled = propAsBool(node, "disabled", false, scope);
        if (placeholder) {
          element.placeholder = placeholder;
        }

        if (stateKey) {
          element.dataset.stateKey = stateKey;
          element.value = String(readState(stateKey, ""));
          element.addEventListener("input", () => {
            runWithEventBoundary("Input.input", node, () => {
              writeState(stateKey, element.value);
              dispatchAction(actionName, element.value, node, scope);
            });
          });
        }
        break;
      }
