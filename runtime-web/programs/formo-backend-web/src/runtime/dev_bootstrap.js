(() => {
  if (typeof window === "undefined") {
    return;
  }

  if (!window.formoState || typeof window.formoState !== "object") {
    window.formoState = {};
  }

  if (!window.formoActions || typeof window.formoActions !== "object") {
    window.formoActions = {};
  }

  if (!("username" in window.formoState)) {
    window.formoState.username = "";
  }

  if (!("showModal" in window.formoState)) {
    window.formoState.showModal = false;
  }

  if (typeof window.formoActions.updateUsername !== "function") {
    window.formoActions.updateUsername = (event) => {
      console.info("[formo] username changed:", event.payload);
    };
  }

  if (typeof window.formoActions.openModal !== "function") {
    window.formoActions.openModal = (event) => {
      event.setState("showModal", true);
    };
  }

  if (typeof window.formoActions.closeModal !== "function") {
    window.formoActions.closeModal = (event) => {
      event.setState("showModal", false);
    };
  }

  if (typeof window.formoActions.toggleModal !== "function") {
    window.formoActions.toggleModal = (event) => {
      event.setState("showModal", Boolean(event.payload));
    };
  }
})();