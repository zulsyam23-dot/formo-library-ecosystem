      case "Button": {
        element = el("button", "fm-button");
        childMount = element;
        element.type = "button";
        element.textContent = propAsString(node, "label", scope) || "Button";
        element.disabled = propAsBool(node, "disabled", false, scope);

        const actionName = propAsString(node, "onPress", scope);
        element.addEventListener("click", () => {
          runWithEventBoundary("Button.click", node, () => {
            dispatchAction(actionName, null, node, scope);
          });
        });
        break;
      }
