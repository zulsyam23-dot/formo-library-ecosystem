      case "Modal": {
        element = el("section", "fm-modal");
        element.setAttribute("role", "dialog");
        element.setAttribute("aria-modal", "true");
        const backdrop = el("div", "fm-modal-backdrop");
        backdrop.setAttribute("aria-hidden", "true");
        const panel = el("div", "fm-modal-panel");
        panel.setAttribute("role", "document");
        panel.tabIndex = -1;
        childMount = panel;

        const stateKey = propAsString(node, "open", scope);
        const isOpen = stateKey ? Boolean(readState(stateKey, false)) : false;
        element.setAttribute("aria-hidden", isOpen ? "false" : "true");
        if (!isOpen) {
          element.classList.add("is-hidden");
        }

        const onClose = propAsString(node, "onClose", scope);
        const closeModal = () => {
          if (stateKey) {
            writeState(stateKey, false);
          }
          dispatchAction(onClose, null, node, scope);
        };
        backdrop.addEventListener("click", () => {
          runWithEventBoundary("Modal.backdrop", node, () => closeModal());
        });

        const closeBtn = el("button", "fm-modal-close");
        closeBtn.type = "button";
        closeBtn.setAttribute("aria-label", "Close modal");
        closeBtn.textContent = "Close";
        closeBtn.addEventListener("click", () => {
          runWithEventBoundary("Modal.closeButton", node, () => closeModal());
        });

        element.addEventListener("keydown", (event) => {
          runWithEventBoundary("Modal.keydown", node, () => {
            if (event.key === "Escape") {
              event.preventDefault();
              closeModal();
              return;
            }
            if (event.key === "Tab") {
              trapTabInContainer(event, panel);
            }
          });
        });

        panel.appendChild(closeBtn);
        element.appendChild(backdrop);
        element.appendChild(panel);
        if (isOpen) {
          queueMicrotask(() => focusFirstInContainer(panel, closeBtn));
        }
        break;
      }
