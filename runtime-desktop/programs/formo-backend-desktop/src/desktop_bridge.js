(() => {
  if (typeof window === "undefined") {
    return;
  }

  const host =
    window.formoDesktopHost && typeof window.formoDesktopHost === "object"
      ? window.formoDesktopHost
      : null;

  if (!window.formoState || typeof window.formoState !== "object") {
    window.formoState = {};
  }

  const baseActions =
    window.formoActions && typeof window.formoActions === "object"
      ? window.formoActions
      : {};

  window.formoActions = new Proxy(baseActions, {
    get(target, prop, receiver) {
      if (Reflect.has(target, prop)) {
        return Reflect.get(target, prop, receiver);
      }

      if (typeof prop !== "string") {
        return Reflect.get(target, prop, receiver);
      }

      if (!host || typeof host.invokeAction !== "function") {
        return undefined;
      }

      return (event) => {
        try {
          host.invokeAction({
            name: prop,
            payload: event && "payload" in event ? event.payload : null,
            nodeId: event && event.nodeId ? event.nodeId : "",
            nodeName: event && event.nodeName ? event.nodeName : "",
            scope: event && event.scope ? event.scope : {},
            state: event && event.state ? event.state : {},
          });
        } catch (err) {
          console.error("[formo-desktop] invokeAction failed", err);
        }
      };
    },
  });

  window.formoDesktop = {
    setStatePatch(patch) {
      if (!patch || typeof patch !== "object") {
        return;
      }
      Object.assign(window.formoState, patch);
    },
    replaceState(next) {
      if (!next || typeof next !== "object") {
        window.formoState = {};
        return;
      }
      window.formoState = { ...next };
    },
  };
})();
