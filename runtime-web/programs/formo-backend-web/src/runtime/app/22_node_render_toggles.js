      case "Checkbox": {
        element = el("label", "fm-check");
        childMount = element;
        const input = el("input", "fm-check-input");
        input.type = "checkbox";
        input.disabled = propAsBool(node, "disabled", false, scope);

        const explicitStateKey = propAsString(node, "checked", scope);
        const stateKey = explicitStateKey || `__local_check::${node.id}`;
        const actionName = propAsString(node, "onChange", scope);
        const initial = propAsBool(node, "checked", false, scope);
        if (!(stateKey in stateStore)) {
          stateStore[stateKey] = initial;
        }
        if (explicitStateKey) {
          input.dataset.stateKey = explicitStateKey;
        }
        input.checked = Boolean(readState(stateKey, initial));
        input.addEventListener("change", () => {
          runWithEventBoundary("Checkbox.change", node, () => {
            writeState(stateKey, input.checked);
            if (explicitStateKey) {
              dispatchAction(actionName, input.checked, node, scope);
            }
          });
        });

        const text = propAsString(node, "label", scope);
        const caption = el("span", "fm-check-label");
        caption.textContent = text || "";
        element.appendChild(input);
        element.appendChild(caption);
        break;
      }
      case "Switch": {
        element = el("label", "fm-switch");
        childMount = element;
        const input = el("input", "fm-switch-input");
        input.type = "checkbox";
        input.disabled = propAsBool(node, "disabled", false, scope);

        const explicitStateKey = propAsString(node, "checked", scope);
        const stateKey = explicitStateKey || `__local_switch::${node.id}`;
        const actionName = propAsString(node, "onChange", scope);
        const initial = propAsBool(node, "checked", false, scope);
        if (!(stateKey in stateStore)) {
          stateStore[stateKey] = initial;
        }
        if (explicitStateKey) {
          input.dataset.stateKey = explicitStateKey;
        }
        input.checked = Boolean(readState(stateKey, initial));
        input.addEventListener("change", () => {
          runWithEventBoundary("Switch.change", node, () => {
            writeState(stateKey, input.checked);
            if (explicitStateKey) {
              dispatchAction(actionName, input.checked, node, scope);
            }
          });
        });

        const text = propAsString(node, "label", scope) || "Switch";
        const caption = el("span", "fm-switch-label");
        caption.textContent = text;
        element.appendChild(input);
        element.appendChild(caption);
        break;
      }
