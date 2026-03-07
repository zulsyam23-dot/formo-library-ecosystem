  function styleIdToClass(styleId) {
    return `fs-${String(styleId).replace(/[^a-zA-Z0-9_-]/g, "-")}`;
  }

  function isIdentifierRef(value) {
    return /^[A-Za-z_][A-Za-z0-9_-]*$/.test(String(value || ""));
  }

  function resolvePathFromRoot(root, parts) {
    let cursor = root;
    for (const part of parts) {
      if (cursor === null || cursor === undefined) {
        return undefined;
      }
      if (typeof cursor !== "object") {
        return undefined;
      }
      if (Array.isArray(cursor)) {
        const index = Number(part);
        if (!Number.isInteger(index) || index < 0 || index >= cursor.length) {
          return undefined;
        }
        cursor = cursor[index];
        continue;
      }
      if (!Object.prototype.hasOwnProperty.call(cursor, part)) {
        return undefined;
      }
      cursor = cursor[part];
    }
    return cursor;
  }