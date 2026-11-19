//! Deterministic CPU-side force layout (Barnes–Hut O(N log N) repulsion).
//!
//! Positions are computed once in Rust and persisted in SQLite so the webview
//! can render massive graphs with the d3 simulation frozen — critical on
//! machines without a dedicated GPU.

use std::collections::HashMap;

use rayon::prelude::*;

use super::db::GraphData;

const NULL: u32 = u32::MAX;
const THETA2: f32 = 0.64; // Barnes–Hut opening angle (0.8) squared
const REPULSION: f32 = 1000.0;
const SOFTENING: f32 = 450.0;
const SPRING_K: f32 = 0.035;
const SPRING_LEN: f32 = 65.0;
const SPRING_LEN_CROSS: f32 = 130.0;
const SPRING_K_CROSS: f32 = 0.015;
const GRAVITY: f32 = 0.001;
const DAMPING: f32 = 0.65;
const MAX_SPEED: f32 = 8.0;
const MAX_DEPTH: u32 = 40;

/// FNV-1a fingerprint over node ids + link endpoints in iteration order.
/// Detects whether persisted positions / cached analytics are still valid.
pub fn graph_fingerprint(graph: &GraphData) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mix = |bytes: &[u8], h: &mut u64| {
        for &b in bytes {
            *h ^= b as u64;
            *h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        *h ^= 0xff;
        *h = h.wrapping_mul(0x0000_0100_0000_01b3);
    };
    for n in &graph.nodes {
        mix(n.id.as_bytes(), &mut h);
    }
    for l in &graph.links {
        mix(l.source.as_bytes(), &mut h);
        mix(l.target.as_bytes(), &mut h);
    }
    h
}

struct QtNode {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    children: [u32; 4],
    point: u32,
    mass: f32,
    cx: f32,
    cy: f32,
}

impl QtNode {
    fn empty(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0,
            y0,
            x1,
            y1,
            children: [NULL; 4],
            point: NULL,
            mass: 0.0,
            cx: 0.0,
            cy: 0.0,
        }
    }

    #[inline]
    fn is_leaf(&self) -> bool {
        self.children[0] == NULL
            && self.children[1] == NULL
            && self.children[2] == NULL
            && self.children[3] == NULL
    }

    #[inline]
    fn quadrant(&self, px: f32, py: f32) -> usize {
        let mx = (self.x0 + self.x1) * 0.5;
        let my = (self.y0 + self.y1) * 0.5;
        ((px >= mx) as usize) | (((py >= my) as usize) << 1)
    }
}

fn subdivide(arena: &mut Vec<QtNode>, n: usize) {
    let (x0, y0, x1, y1) = (arena[n].x0, arena[n].y0, arena[n].x1, arena[n].y1);
    let mx = (x0 + x1) * 0.5;
    let my = (y0 + y1) * 0.5;
    let base = arena.len() as u32;
    arena.push(QtNode::empty(x0, y0, mx, my));
    arena.push(QtNode::empty(mx, y0, x1, my));
    arena.push(QtNode::empty(x0, my, mx, y1));
    arena.push(QtNode::empty(mx, my, x1, y1));
    arena[n].children = [base, base + 1, base + 2, base + 3];
}

fn insert(
    arena: &mut Vec<QtNode>,
    pos: &[(f32, f32)],
    n: usize,
    idx: u32,
    depth: u32,
) {
    let (px, py) = pos[idx as usize];
    {
        let node = &mut arena[n];
        node.mass += 1.0;
        node.cx += (px - node.cx) / node.mass;
        node.cy += (py - node.cy) / node.mass;
    }
    if arena[n].is_leaf() {
        if arena[n].point == NULL {
            arena[n].point = idx;
            return;
        }
        if depth >= MAX_DEPTH {
            return; // coincident points — accept the collision
        }
        let old = arena[n].point;
        arena[n].point = NULL;
        subdivide(arena, n);
        // Re-insert the displaced point and the new one into the correct
        // children — NOT through `n`, whose mass was already accounted.
        let (ox, oy) = pos[old as usize];
        let oq = arena[n].quadrant(ox, oy);
        insert(arena, pos, arena[n].children[oq] as usize, old, depth + 1);
        let q = arena[n].quadrant(px, py);
        insert(arena, pos, arena[n].children[q] as usize, idx, depth + 1);
        return;
    }
    let q = arena[n].quadrant(px, py);
    let child = arena[n].children[q];
    insert(arena, pos, child as usize, idx, depth + 1);
}

fn repulse(
    arena: &[QtNode],
    root: usize,
    idx: u32,
    px: f32,
    py: f32,
    alpha: f32,
    stack: &mut Vec<usize>,
) -> (f32, f32) {
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;
    stack.clear();
    stack.push(root);
    while let Some(n) = stack.pop() {
        let node = &arena[n];
        if node.mass <= 0.0 {
            continue;
        }
        let dx = px - node.cx;
        let dy = py - node.cy;
        let d2 = dx * dx + dy * dy;
        if node.is_leaf() {
            if node.point != NULL && node.point != idx {
                let d = d2.sqrt().max(0.1);
                let f = REPULSION * alpha / (d2 + SOFTENING);
                fx += (dx / d) * f;
                fy += (dy / d) * f;
            }
            continue;
        }
        let w = node.x1 - node.x0;
        if w * w < THETA2 * d2 {
            let d = d2.sqrt().max(0.1);
            let f = REPULSION * node.mass * alpha / (d2 + SOFTENING);
            fx += (dx / d) * f;
            fy += (dy / d) * f;
        } else {
            for &c in &node.children {
                if c != NULL {
                    stack.push(c as usize);
                }
            }
        }
    }
    (fx, fy)
}

/// Deterministic community-clustered force layout.
/// `communities` is aligned to `graph.nodes`; `usize::MAX` = unassigned.
/// Returns positions aligned to `graph.nodes` order.
pub fn compute_layout(graph: &GraphData, communities: &[usize]) -> Vec<(f32, f32)> {
    let n = graph.nodes.len();
    if n == 0 {
        return Vec::new();
    }

    // --- Initial placement: communities on a circle, golden-angle jitter ---
    let comm_count = communities
        .iter()
        .filter(|&&c| c != usize::MAX)
        .max()
        .map(|m| m + 1)
        .unwrap_or(1)
        .max(1);
    let cluster_radius = (n as f32).sqrt() * 38.0;
    let mut pos: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let c = communities.get(i).copied().unwrap_or(usize::MAX);
        let c_eff = if c == usize::MAX { (i % comm_count) as usize } else { c };
        let cluster_angle = (c_eff as f32 / comm_count as f32) * std::f32::consts::TAU;
        let cx = cluster_angle.cos() * cluster_radius;
        let cy = cluster_angle.sin() * cluster_radius;
        let jitter_angle = i as f32 * 2.399_963_2; // golden angle — deterministic
        let jitter_dist = 25.0 + (i % 7) as f32 * 12.0;
        pos.push((
            cx + jitter_angle.cos() * jitter_dist,
            cy + jitter_angle.sin() * jitter_dist,
        ));
    }

    // --- Edge index pairs ---
    let id_to_idx: HashMap<&str, u32> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i as u32))
        .collect();
    let edges: Vec<(u32, u32)> = graph
        .links
        .iter()
        .filter_map(|l| {
            Some((*id_to_idx.get(l.source.as_str())?, *id_to_idx.get(l.target.as_str())?))
        })
        .collect();

    let ticks = if n <= 3_000 {
        240
    } else if n <= 30_000 {
        160
    } else if n <= 150_000 {
        110
    } else {
        80
    };

    let mut vel = vec![(0.0f32, 0.0f32); n];
    let mut alpha = 1.0f32;
    const ALPHA_DECAY: f32 = 0.035;
    const ALPHA_MIN: f32 = 0.02;

    for _ in 0..ticks {
        if alpha <= ALPHA_MIN {
            break;
        }

        // 1. Build quadtree over current positions
        let (mut bx0, mut by0, mut bx1, mut by1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &(x, y) in &pos {
            bx0 = bx0.min(x);
            by0 = by0.min(y);
            bx1 = bx1.max(x);
            by1 = by1.max(y);
        }
        // Square bounds + pad to avoid zero-width degenerate trees
        let cx = (bx0 + bx1) * 0.5;
        let cy = (by0 + by1) * 0.5;
        let half = ((bx1 - bx0).max(by1 - by0) * 0.5).max(1.0) + 1.0;
        let mut arena: Vec<QtNode> = Vec::with_capacity(n / 2 + 16);
        arena.push(QtNode::empty(cx - half, cy - half, cx + half, cy + half));
        for i in 0..n {
            insert(&mut arena, &pos, 0, i as u32, 0);
        }

        // 2. Repulsion (Barnes–Hut), parallel over nodes
        let arena = &arena;
        let pos_ref = &pos;
        let forces: Vec<(f32, f32)> = (0..n)
            .into_par_iter()
            .map_init(Vec::new, |stack, i| {
                let (px, py) = pos_ref[i];
                repulse(arena, 0, i as u32, px, py, alpha, stack)
            })
            .collect();

        // 3. Apply repulsion + springs + gravity into velocities
        for (i, &(fx, fy)) in forces.iter().enumerate() {
            vel[i].0 += fx;
            vel[i].1 += fy;
        }
        for &(s, t) in &edges {
            let (sx, sy) = pos[s as usize];
            let (tx, ty) = pos[t as usize];
            let dx = tx - sx;
            let dy = ty - sy;
            let dist = (dx * dx + dy * dy).sqrt().max(0.1);
            let cross = communities
                .get(s as usize)
                .zip(communities.get(t as usize))
                .map(|(&a, &b)| a != usize::MAX && b != usize::MAX && a != b)
                .unwrap_or(false);
            let (desired, k) = if cross {
                (SPRING_LEN_CROSS, SPRING_K_CROSS)
            } else {
                (SPRING_LEN, SPRING_K)
            };
            let f = (dist - desired) * k * alpha;
            let fx = (dx / dist) * f;
            let fy = (dy / dist) * f;
            vel[s as usize].0 += fx;
            vel[s as usize].1 += fy;
            vel[t as usize].0 -= fx;
            vel[t as usize].1 -= fy;
        }

        // 4. Integrate
        for i in 0..n {
            let (px, py) = pos[i];
            let (mut vx, mut vy) = vel[i];
            vx += -px * GRAVITY * alpha;
            vy += -py * GRAVITY * alpha;
            vx *= DAMPING;
            vy *= DAMPING;
            let speed = (vx * vx + vy * vy).sqrt();
            if speed > MAX_SPEED {
                vx = vx / speed * MAX_SPEED;
                vy = vy / speed * MAX_SPEED;
            }
            pos[i] = (px + vx, py + vy);
            vel[i] = (vx, vy);
        }

        alpha -= ALPHA_DECAY * alpha;
    }

    pos
}
