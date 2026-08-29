/** Blokuje skróty przeglądarki / DevTools; F3 otwiera menu Octra. */
export function installKeyboardGuard(onF3: () => void): () => void {
  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "F3") {
      e.preventDefault();
      e.stopPropagation();
      onF3();
      return;
    }

    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;
    const key = e.key.toLowerCase();

    const devtools =
      e.key === "F12" ||
      (ctrl && shift && (key === "i" || key === "j" || key === "c")) ||
      (ctrl && key === "u");

    const reload = e.key === "F5" || (ctrl && key === "r");

    if (devtools || reload || e.key === "F11") {
      e.preventDefault();
      e.stopPropagation();
    }
  }

  function onContextMenu(e: Event) {
    e.preventDefault();
  }

  window.addEventListener("keydown", onKeyDown, true);
  window.addEventListener("contextmenu", onContextMenu, true);
  return () => {
    window.removeEventListener("keydown", onKeyDown, true);
    window.removeEventListener("contextmenu", onContextMenu, true);
  };
}
