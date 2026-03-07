(() => {
  const irScript = document.getElementById("formo-ir");
  const mount = document.getElementById("app");
  if (!irScript || !mount) {
    return;
  }

  let ir;
  try {
    ir = JSON.parse(irScript.textContent || "{}");
  } catch (err) {
    mount.innerHTML = `<pre class="fm-error">IR parse failed: ${String(err)}</pre>`;
    return;
  }

  const nodes = new Map((ir.nodes || []).map((node) => [node.id, node]));
  const components = ir.components || [];
  const entry = components.find((comp) => comp.name === ir.entry) || components[0];

  if (!entry) {
    mount.innerHTML = '<div class="fm-error">No component found in IR.</div>';
    return;
  }

  const actionHandlers =
    typeof window !== "undefined" &&
    window.formoActions &&
    typeof window.formoActions === "object"
      ? window.formoActions
      : {};

  const stateStore =
    typeof window !== "undefined" &&
    window.formoState &&
    typeof window.formoState === "object"
      ? window.formoState
      : {};

  if (typeof window !== "undefined") {
    window.formoState = stateStore;
    if (!Array.isArray(window.formoRuntimeErrors)) {
      window.formoRuntimeErrors = [];
    }
  }

  initializeStateDefaults();
  const stateUsage = buildStateUsageMap();

  let renderQueued = false;
  let suppressForBindingRegistration = false;
  const forBindingsByStateKey = new Map();
  render();
