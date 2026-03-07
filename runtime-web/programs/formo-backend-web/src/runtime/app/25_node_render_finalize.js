      case "Fragment":
        element = document.createDocumentFragment();
        childMount = element;
        break;
      default:
        element = el("div", "fm-node");
        childMount = element;
        element.setAttribute("data-formo-node", node.name);
        break;
    }

    if (
      element.nodeType === Node.ELEMENT_NODE &&
      Array.isArray(node.styleRefs) &&
      node.styleRefs.length > 0
    ) {
      element.setAttribute("data-style-refs", node.styleRefs.join(" "));
      for (const ref of node.styleRefs) {
        element.classList.add(styleIdToClass(ref));
      }
    }

    const childIds = Array.isArray(node.children) ? node.children : [];
    for (const childId of childIds) {
      childMount.appendChild(buildNode(childId, nodeMap, scope));
    }

    return element;
  }