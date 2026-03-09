  function deriveForItems(node, scope) {
    const eachValue = propAsList(node, "each", scope);
    if (eachValue.length > 0) {
      return eachValue;
    }

    const countText = String(propAsString(node, "count", scope) || "").trim();
    if (!countText) {
      return [];
    }

    const parsed = Number(countText);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      return [];
    }

    const count = Math.floor(parsed);
    const out = [];
    for (let i = 0; i < count; i += 1) {
      out.push(i);
    }
    return out;
  }

  function renderForItemsIntoContainer(container, childIds, alias, scope, eachValue, nodeMap) {
    const wrappers = [];
    for (let i = 0; i < eachValue.length; i += 1) {
      const item = eachValue[i];
      const wrapper = createForItemWrapper();
      wrapper.dataset.fmForKey = deriveForItemKey(item, i);
      renderForItemIntoWrapper(wrapper, childIds, alias, scope, item, i, nodeMap);
      wrappers.push(wrapper);
    }

    container.replaceChildren(...wrappers);
  }

  function renderForItemIntoWrapper(wrapper, childIds, alias, scope, item, index, nodeMap) {
    const nextScope = { ...(scope || {}) };
    nextScope[alias] = item;
    nextScope[`${alias}Index`] = index;
    nextScope[`${alias}Key`] = deriveForItemKey(item, index);

    const children = [];
    for (const childId of childIds) {
      children.push(buildNode(childId, nodeMap, nextScope));
    }
    wrapper.replaceChildren(...children);
  }

  function createForItemWrapper() {
    const wrapper = el("div", "fm-for-item");
    wrapper.style.display = "contents";
    return wrapper;
  }

  function deriveForItemKey(item, index) {
    if (item && typeof item === "object") {
      if (item.id !== undefined && item.id !== null) {
        return `id:${String(item.id)}`;
      }
      if (item.key !== undefined && item.key !== null) {
        return `key:${String(item.key)}`;
      }
    }

    return `idx:${index}`;
  }
