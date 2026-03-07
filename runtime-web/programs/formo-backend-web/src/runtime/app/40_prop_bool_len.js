  function propAsBool(node, key, fallback, scope) {
    const prop = propRecord(node, key);
    if (!prop) {
      return fallback;
    }

    const resolved = resolveScopedValue(prop, scope);
    if (typeof resolved === "boolean") {
      return resolved;
    }

    if (prop.t === "bool" && resolved !== undefined) {
      return Boolean(resolved);
    }

    if (typeof resolved === "string") {
      if (resolved === "true") {
        return true;
      }
      if (resolved === "false") {
        return false;
      }
      if (resolved in stateStore) {
        return Boolean(stateStore[resolved]);
      }
    }

    if (typeof resolved === "number") {
      return resolved !== 0;
    }

    return fallback;
  }

  function propAsLen(node, key, scope) {
    const prop = propRecord(node, key);
    if (!prop) {
      return "";
    }

    const resolved = resolveScopedValue(prop, scope);

    if (prop.t === "len" && resolved && typeof resolved === "object") {
      const value = Number(resolved.value ?? 0);
      const unit = String(resolved.unit ?? "px");
      const cssUnit = unit === "dp" ? "px" : unit;
      return `${value}${cssUnit}`;
    }

    if ((prop.t === "int" || prop.t === "float") && typeof resolved === "number") {
      return `${resolved}px`;
    }

    if (typeof resolved === "string") {
      return resolved;
    }

    return "";
  }