  function propRecord(node, key) {
    return node.props && node.props[key] ? node.props[key] : null;
  }

  function propAsString(node, key, scope) {
    const prop = propRecord(node, key);
    if (!prop) {
      return "";
    }

    const resolved = resolveScopedValue(prop, scope);
    if (resolved === null || resolved === undefined) {
      return "";
    }

    return String(resolved);
  }

  function propLiteralString(node, key) {
    const prop = propRecord(node, key);
    if (
      !prop ||
      prop.t !== "string" ||
      prop.v === null ||
      prop.v === undefined ||
      typeof prop.v !== "string"
    ) {
      return "";
    }
    return prop.v;
  }