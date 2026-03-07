      case "If": {
        element = document.createDocumentFragment();
        childMount = element;
        const when = propAsBool(node, "when", false, scope);
        if (!when) {
          return element;
        }
        break;
      }
      case "For": {
        element = el("div", "fm-for");
        element.style.display = "contents";
        childMount = element;

        const eachValue = propAsList(node, "each", scope);
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