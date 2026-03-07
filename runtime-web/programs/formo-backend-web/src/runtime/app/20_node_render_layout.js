  function buildNode(nodeId, nodeMap, scope) {
    const node = nodeMap.get(nodeId);
    if (!node) {
      return errorEl(`Unknown node id: ${nodeId}`);
    }

    let element;
    let childMount;

    switch (node.name) {
      case "Window": {
        element = el("section", "fm-window");
        childMount = element;
        const title = propAsString(node, "title", scope);
        if (title) {
          const head = el("h1", "fm-window-title");
          head.textContent = title;
          element.appendChild(head);
        }
        break;
      }
      case "Page":
        element = el("main", "fm-page");
        childMount = element;
        break;
      case "Row":
        element = el("div", "fm-row");
        childMount = element;
        break;
      case "Column":
        element = el("div", "fm-column");
        childMount = element;
        break;
      case "Stack":
        element = el("div", "fm-stack");
        childMount = element;
        break;
      case "Card":
        element = el("section", "fm-card");
        childMount = element;
        break;
      case "Scroll":
        element = el("div", "fm-scroll");
        childMount = element;
        break;
      case "Text":
        element = el("span", "fm-text");
        childMount = element;
        element.textContent = propAsString(node, "value", scope) || "";
        break;
      case "Image": {
        element = el("img", "fm-image");
        childMount = element;
        const src = propAsString(node, "src", scope);
        const alt = propAsString(node, "alt", scope);
        if (src) {
          element.src = src;
        }
        element.alt = alt;

        const width = propAsLen(node, "width", scope);
        if (width) {
          element.style.width = width;
        }

        const height = propAsLen(node, "height", scope);
        if (height) {
          element.style.height = height;
        }
        break;
      }
      case "Spacer": {
        element = el("div", "fm-spacer");
        childMount = element;
        const size = propAsLen(node, "size", scope);
        if (size) {
          element.style.minHeight = size;
          element.style.minWidth = size;
        }
        break;
      }