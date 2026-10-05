(() => {
  const messageType = 'gh-mdp-app-zoom';
  const levels = [0.5, 0.67, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2];

  const zoomDirection = event => {
    if (!event.ctrlKey || event.metaKey || event.altKey) return 0;
    if (event.key === '+' || event.key === '=') return 1;
    if (event.key === '-') return -1;
    if (event.key === '0') return Infinity;
    return 0;
  };

  if (window === window.top) {
    const storageKey = 'gh-mdp-app-zoom-level';
    const savedLevel = Number(sessionStorage.getItem(storageKey));
    let levelIndex = levels.indexOf(savedLevel);
    if (levelIndex < 0) levelIndex = levels.indexOf(1);

    const applyZoom = () => {
      document.documentElement.style.zoom = levels[levelIndex];
    };
    const changeZoom = direction => {
      if (direction === Infinity) {
        levelIndex = levels.indexOf(1);
      } else {
        levelIndex = Math.max(0, Math.min(levels.length - 1, levelIndex + direction));
      }
      sessionStorage.setItem(storageKey, levels[levelIndex]);
      applyZoom();
    };
    const handleKeydown = event => {
      const direction = zoomDirection(event);
      if (!direction) return;
      event.preventDefault();
      changeZoom(direction);
    };

    if (document.documentElement) applyZoom();
    else document.addEventListener('DOMContentLoaded', applyZoom, { once: true });
    document.addEventListener('keydown', handleKeydown);
    window.addEventListener('message', event => {
      if (event.origin !== location.origin || event.data?.type !== messageType) return;
      changeZoom(event.data.direction);
    });
  } else {
    document.addEventListener('keydown', event => {
      const direction = zoomDirection(event);
      if (!direction) return;
      event.preventDefault();
      window.top.postMessage({ type: messageType, direction }, location.origin);
    });
  }
})();
