/** The board window survives closing so its geometry remains available. */
export function createReferenceBoardDragSession() {
  let dragElsewhereOnce = false;
  return {
    async nextDrag(window) {
      if (window && await window.isVisible()) return 'native';
      if (dragElsewhereOnce) {
        dragElsewhereOnce = false;
        return 'native';
      }
      return 'prompt';
    },
    chooseExternal() { dragElsewhereOnce = true; },
    boardClosed() { dragElsewhereOnce = false; },
  };
}
