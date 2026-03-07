  function registerForBinding(stateKey, binding) {
    if (suppressForBindingRegistration || !stateKey || !binding || !binding.container) {
      return;
    }

    if (!forBindingsByStateKey.has(stateKey)) {
      forBindingsByStateKey.set(stateKey, []);
    }
    forBindingsByStateKey.get(stateKey).push(binding);
  }

  function updateForBindingsForKey(stateKey) {
    const bindings = forBindingsByStateKey.get(stateKey);
    if (!bindings || bindings.length === 0) {
      return false;
    }

    let updated = false;
    for (const binding of bindings) {
      if (!binding || !binding.container || !binding.container.isConnected) {
        continue;
      }
      updateForBinding(binding, stateKey);
      updated = true;
    }

    return updated;
  }

  function updateForBinding(binding, stateKey) {
    const listValue = Array.isArray(stateStore[stateKey]) ? stateStore[stateKey] : [];
    const wrapperPool = new Map();
    for (const child of Array.from(binding.container.children)) {
      const wrapperKey =
        child && child.dataset && typeof child.dataset.fmForKey === "string"
          ? child.dataset.fmForKey
          : "";
      if (!wrapperPool.has(wrapperKey)) {
        wrapperPool.set(wrapperKey, []);
      }
      wrapperPool.get(wrapperKey).push(child);
    }

    const nextWrappers = [];
    suppressForBindingRegistration = true;
    try {
      for (let i = 0; i < listValue.length; i += 1) {
        const item = listValue[i];
        const stableKey = deriveForItemKey(item, i);
        const fromPool = wrapperPool.get(stableKey);
        const wrapper = fromPool && fromPool.length > 0 ? fromPool.shift() : createForItemWrapper();
        wrapper.dataset.fmForKey = stableKey;
        renderForItemIntoWrapper(
          wrapper,
          binding.childIds,
          binding.alias,
          binding.baseScope,
          item,
          i,
          nodes
        );
        nextWrappers.push(wrapper);
      }
    } finally {
      suppressForBindingRegistration = false;
    }

    binding.container.replaceChildren(...nextWrappers);
  }
