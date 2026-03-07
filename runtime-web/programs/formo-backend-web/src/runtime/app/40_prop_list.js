  function propAsList(node, key, scope) {
    const prop = propRecord(node, key);
    if (!prop) {
      return [];
    }

    const resolved = resolveScopedValue(prop, scope);
    if (Array.isArray(resolved)) {
      return resolved;
    }

    if (typeof resolved === "string") {
      const text = resolved.trim();
      if (!text) {
        return [];
      }

      if (text in stateStore && Array.isArray(stateStore[text])) {
        return stateStore[text];
      }

      if (text.startsWith("[") && text.endsWith("]")) {
        try {
          const parsed = JSON.parse(text);
          return Array.isArray(parsed) ? parsed : [];
        } catch (_) {
          return [];
        }
      }
    }

    return [];
  }