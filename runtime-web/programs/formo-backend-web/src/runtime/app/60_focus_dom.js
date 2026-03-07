  function captureFocusSnapshot() {
    const active = document.activeElement;
    if (!active || !active.dataset || !active.dataset.stateKey) {
      return null;
    }

    return {
      stateKey: active.dataset.stateKey,
      selectionStart:
        typeof active.selectionStart === "number" ? active.selectionStart : null,
      selectionEnd:
        typeof active.selectionEnd === "number" ? active.selectionEnd : null,
    };
  }

  function getFocusableElements(container) {
    if (!container || typeof container.querySelectorAll !== "function") {
      return [];
    }

    const selector =
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
    return Array.from(container.querySelectorAll(selector));
  }

  function focusFirstInContainer(container, fallback) {
    const focusables = getFocusableElements(container);
    const target = focusables[0] || fallback || container;
    if (target && typeof target.focus === "function") {
      target.focus();
    }
  }

  function trapTabInContainer(event, container) {
    if (!event || event.key !== "Tab") {
      return;
    }

    const focusables = getFocusableElements(container);
    if (focusables.length === 0) {
      event.preventDefault();
      return;
    }

    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    const active = document.activeElement;
    const inside = container && typeof container.contains === "function" && container.contains(active);

    if (event.shiftKey) {
      if (!inside || active === first) {
        event.preventDefault();
        last.focus();
      }
      return;
    }

    if (!inside || active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function restoreFocusSnapshot(snapshot) {
    if (!snapshot) {
      return;
    }

    const candidates = document.querySelectorAll("[data-state-key]");
    for (const candidate of candidates) {
      if (candidate.dataset.stateKey !== snapshot.stateKey) {
        continue;
      }

      if (typeof candidate.focus === "function") {
        candidate.focus();
      }

      if (
        snapshot.selectionStart !== null &&
        snapshot.selectionEnd !== null &&
        typeof candidate.setSelectionRange === "function"
      ) {
        try {
          candidate.setSelectionRange(snapshot.selectionStart, snapshot.selectionEnd);
        } catch (_) {
          // Ignore selection errors for controls that do not support caret range.
        }
      }

      break;
    }
  }

  function el(tag, className) {
    const node = document.createElement(tag);
    if (className) {
      node.className = className;
    }
    return node;
  }

  function errorEl(message) {
    const node = el("div", "fm-error");
    node.textContent = message;
    return node;
  }
})();
