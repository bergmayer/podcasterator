type PhysicalPoint = {
  x: number;
  y: number;
};

export function isPositionInside(
  position: PhysicalPoint,
  element: HTMLElement | null
): boolean {
  if (!element) return false;

  // Tauri reports native drag positions in physical pixels, while DOM bounds
  // use logical CSS pixels.
  const scaleFactor = window.devicePixelRatio || 1;
  const x = position.x / scaleFactor;
  const y = position.y / scaleFactor;
  const bounds = element.getBoundingClientRect();

  return (
    x >= bounds.left &&
    x <= bounds.right &&
    y >= bounds.top &&
    y <= bounds.bottom
  );
}
