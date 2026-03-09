      case "If": {
        element = document.createDocumentFragment();
        childMount = element;
        const hasWhen = Boolean(propRecord(node, "when"));
        const hasCondition = Boolean(propRecord(node, "condition"));
        const shouldShow = hasWhen
          ? propAsBool(node, "when", false, scope)
          : hasCondition
            ? propAsBool(node, "condition", false, scope)
            : false;
        if (!shouldShow) {
          return element;
        }
        break;
      }
      case "For": {
        element = el("div", "fm-for");
        element.style.display = "contents";
        childMount = element;

        const eachValue = deriveForItems(node, scope);
        const alias = propLiteralString(node, "as") || "item";
        const childIds = Array.isArray(node.children) ? node.children : [];
        const eachStateKey = forEachStateKey(node, scope);

        renderForItemsIntoContainer(
          element,
          childIds,
          alias,
          scope,
          eachValue,
          nodeMap
        );

        if (eachStateKey) {
          registerForBinding(eachStateKey, {
            container: element,
            childIds,
            alias,
            baseScope: { ...(scope || {}) },
          });
        }

        return element;
      }
