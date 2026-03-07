      case "Checkbox": {
        element = el("label", "fm-check");
        childMount = element;
        const input = el("input", "fm-check-input");
        input.type = "checkbox";
        input.disabled = propAsBool(node, "disabled", false, scope);

        const stateKey = propAsString(node, "checked", scope);
        const actionName = propAsString(node, "onChange", scope);
        if (stateKey) {
          input.dataset.stateKey = stateKey;
          input.checked = Boolean(readState(stateKey, false));
          input.addEventListener("change", () => {
            runWithEventBoundary("Checkbox.change", node, () => {
              writeState(stateKey, input.checked);
              dispatchAction(actionName, input.checked, node, scope);
            });
          });
        }

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

        const stateKey = propAsString(node, "checked", scope);
        const actionName = propAsString(node, "onChange", scope);
        if (stateKey) {
          input.dataset.stateKey = stateKey;
          input.checked = Boolean(readState(stateKey, false));
          input.addEventListener("change", () => {
            runWithEventBoundary("Switch.change", node, () => {
              writeState(stateKey, input.checked);
              dispatchAction(actionName, input.checked, node, scope);
            });
          });
        }

        const slider = el("span", "fm-switch-slider");
        element.appendChild(input);
        element.appendChild(slider);
        break;
      }
