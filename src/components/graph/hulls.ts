export interface Point2D {
  x: number;
  y: number;
}

export function computeConvexHull(points: Point2D[]): Point2D[] {
  if (points.length <= 2) return points.slice();
  const sorted = points
    .slice()
    .sort((a, b) => (a.x !== b.x ? a.x - b.x : a.y - b.y));

  const cross = (o: Point2D, a: Point2D, b: Point2D) =>
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

  const lower: Point2D[] = [];
  for (const p of sorted) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], p) <= 0) {
      lower.pop();
    }
    lower.push(p);
  }

  const upper: Point2D[] = [];
  for (let i = sorted.length - 1; i >= 0; i--) {
    const p = sorted[i];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], p) <= 0) {
      upper.pop();
    }
    upper.push(p);
  }

  upper.pop();
  lower.pop();
  return lower.concat(upper);
}

export function drawSmoothHull(ctx: CanvasRenderingContext2D, hull: Point2D[]) {
  if (hull.length < 3) return;
  ctx.beginPath();
  const len = hull.length;
  const firstMidX = (hull[len - 1].x + hull[0].x) / 2;
  const firstMidY = (hull[len - 1].y + hull[0].y) / 2;
  ctx.moveTo(firstMidX, firstMidY);
  for (let i = 0; i < len; i++) {
    const p1 = hull[i];
    const p2 = hull[(i + 1) % len];
    const midX = (p1.x + p2.x) / 2;
    const midY = (p1.y + p2.y) / 2;
    ctx.quadraticCurveTo(p1.x, p1.y, midX, midY);
  }
  ctx.closePath();
}
