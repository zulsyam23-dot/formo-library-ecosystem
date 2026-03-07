  function resolveScopedValue(prop, scope) {
    if (!prop) {
      return undefined;
    }

    if (prop.t !== "string") {
      return prop.v;
    }

    if (typeof prop.v !== "string") {
      return prop.v;
    }

    if (!scope || typeof scope !== "object") {
      return prop.v;
    }

    if (Object.prototype.hasOwnProperty.call(scope, prop.v)) {
      return scope[prop.v];
    }

    if (typeof prop.v === "string" && prop.v.includes(".")) {
      const pathParts = prop.v.split(".").filter((piece) => piece.length > 0);
      if (pathParts.length > 0) {
        const baseKey = pathParts[0];
        if (Object.prototype.hasOwnProperty.call(scope, baseKey)) {
          const scoped = resolvePathFromRoot(scope[baseKey], pathParts.slice(1));
          if (scoped !== undefined) {
            return scoped;
          }
        }
      }
    }

    return prop.v;
  }