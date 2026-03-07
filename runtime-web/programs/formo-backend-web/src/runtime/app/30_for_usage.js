  function buildStateUsageMap() {
    const usageMap = new Map();
    for (const node of nodes.values()) {
      if (!node || !node.props || typeof node.props !== "object") {
        continue;
      }

      for (const [propName, prop] of Object.entries(node.props)) {
        if (!prop || prop.t !== "string" || typeof prop.v !== "string") {
          continue;
        }

        const ref = prop.v.trim();
        if (!isIdentifierRef(ref)) {
          continue;
        }

        let usage = usageMap.get(ref);
        if (!usage) {
          usage = { forEach: 0, other: 0 };
          usageMap.set(ref, usage);
        }

        if (node.name === "For" && propName === "each") {
          usage.forEach += 1;
        } else {
          usage.other += 1;
        }
      }
    }

    return usageMap;
  }

  function canPatchForStateKey(key, value) {
    if (!key || !Array.isArray(value)) {
      return false;
    }

    const usage = stateUsage.get(key);
    return Boolean(usage && usage.forEach > 0 && usage.other === 0);
  }

  function forEachStateKey(node, scope) {
    const each = propRecord(node, "each");
    if (!each || each.t !== "string" || typeof each.v !== "string") {
      return "";
    }

    const raw = each.v.trim();
    if (!isIdentifierRef(raw)) {
      return "";
    }

    if (scope && Object.prototype.hasOwnProperty.call(scope, raw)) {
      return "";
    }

    return raw;
  }
