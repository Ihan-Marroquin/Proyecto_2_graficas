use nalgebra_glm as glm;
use crate::cube::Cube;
use crate::ray_intersect::Intersect;
use crate::ray_intersect::RayIntersect;

// Tipo de nodo BVH
#[derive(Debug)]
pub enum BVHNode {
    Leaf { bbox_min: glm::Vec3, bbox_max: glm::Vec3, start: usize, count: usize },
    Node { bbox_min: glm::Vec3, bbox_max: glm::Vec3, left: Box<BVHNode>, right: Box<BVHNode> },
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct BVH {
    pub root: BVHNode,
    pub indices: Vec<usize>,
}

// Unión de dos AABBs
fn union_bbox(a_min: &glm::Vec3, a_max: &glm::Vec3, b_min: &glm::Vec3, b_max: &glm::Vec3) -> (glm::Vec3, glm::Vec3) {
    let min = glm::vec3(a_min.x.min(b_min.x), a_min.y.min(b_min.y), a_min.z.min(b_min.z));
    let max = glm::vec3(a_max.x.max(b_max.x), a_max.y.max(b_max.y), a_max.z.max(b_max.z));
    (min, max)
}

// Calcular bbox para un conjunto de objetos
fn bbox_for_indices(objects: &[Cube], indices: &[usize], start: usize, count: usize) -> (glm::Vec3, glm::Vec3) {
    let mut bmin = glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut bmax = glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for i in start..start+count {
        let obj = &objects[indices[i]];
        // cube extents
        let ex = obj.half_size.x; let ey = obj.half_size.y; let ez = obj.half_size.z;
        let min = glm::vec3(obj.center.x - ex, obj.center.y - ey, obj.center.z - ez);
        let max = glm::vec3(obj.center.x + ex, obj.center.y + ey, obj.center.z + ez);
        bmin = glm::vec3(bmin.x.min(min.x), bmin.y.min(min.y), bmin.z.min(min.z));
        bmax = glm::vec3(bmax.x.max(max.x), bmax.y.max(max.y), bmax.z.max(max.z));
    }
    (bmin, bmax)
}

// Devolver el eje más largo de bbox
fn longest_axis(min: &glm::Vec3, max: &glm::Vec3) -> usize {
    let diag = *max - *min;
    if diag.x > diag.y && diag.x > diag.z { 0 } else if diag.y > diag.z { 1 } else { 2 }
}

// División por mediana (placeholder, no usado)
#[allow(dead_code)]
fn median_split(_objects: &mut [usize], _axis: usize) {
    // placeholder: no usado actualmente
}

pub fn build_bvh(objects: &[Cube]) -> BVH {
    let mut indices: Vec<usize> = (0..objects.len()).collect();
    let n = indices.len();
    let root = build_recursive(objects, &mut indices[..], 0, n);
    BVH { root, indices }
}

fn build_recursive(objects: &[Cube], indices: &mut [usize], start: usize, count: usize) -> BVHNode {
    let (bmin, bmax) = bbox_for_indices(objects, indices, start, count);
    if count <= 8 {
        return BVHNode::Leaf { bbox_min: bmin, bbox_max: bmax, start, count };
    }
    let mut cmin = glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut cmax = glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    let _centroids: Vec<(usize, f32)> = Vec::with_capacity(count);
    for i in start..start+count {
        let obj = &objects[indices[i]];
        let c = obj.center;
        cmin = glm::vec3(cmin.x.min(c.x), cmin.y.min(c.y), cmin.z.min(c.z));
        cmax = glm::vec3(cmax.x.max(c.x), cmax.y.max(c.y), cmax.z.max(c.z));
    }
    let axis = longest_axis(&cmin, &cmax);
    indices[start..start+count].sort_by(|&ia, &ib| {
        let ca = match axis { 0 => objects[ia].center.x, 1 => objects[ia].center.y, _ => objects[ia].center.z };
        let cb = match axis { 0 => objects[ib].center.x, 1 => objects[ib].center.y, _ => objects[ib].center.z };
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = start + count / 2;
    let left = build_recursive(objects, indices, start, mid - start);
    let right = build_recursive(objects, indices, mid, start + count - mid);
    let (lmin, lmax) = match &left { BVHNode::Leaf{bbox_min, bbox_max, ..} => (*bbox_min, *bbox_max), BVHNode::Node{bbox_min, bbox_max, ..} => (*bbox_min, *bbox_max) };
    let (rmin, rmax) = match &right { BVHNode::Leaf{bbox_min, bbox_max, ..} => (*bbox_min, *bbox_max), BVHNode::Node{bbox_min, bbox_max, ..} => (*bbox_min, *bbox_max) };
    let (nbmin, nbmax) = union_bbox(&lmin, &lmax, &rmin, &rmax);
    BVHNode::Node { bbox_min: nbmin, bbox_max: nbmax, left: Box::new(left), right: Box::new(right) }
}

// Intersección rayo-AABB (método slab) con inversos seguros
pub fn ray_intersect_aabb(orig: &glm::Vec3, dir: &glm::Vec3, bmin: &glm::Vec3, bmax: &glm::Vec3) -> Option<(f32,f32)> {
    let inv_dx = if dir.x.abs() < 1e-8 { f32::INFINITY } else { 1.0 / dir.x };
    let inv_dy = if dir.y.abs() < 1e-8 { f32::INFINITY } else { 1.0 / dir.y };
    let inv_dz = if dir.z.abs() < 1e-8 { f32::INFINITY } else { 1.0 / dir.z };

    let mut tmin = (bmin.x - orig.x) * inv_dx;
    let mut tmax = (bmax.x - orig.x) * inv_dx;
    if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }
    let mut tymin = (bmin.y - orig.y) * inv_dy;
    let mut tymax = (bmax.y - orig.y) * inv_dy;
    if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }
    if (tmin > tymax) || (tymin > tmax) { return None; }
    if tymin > tmin { tmin = tymin; }
    if tymax < tmax { tmax = tymax; }
    let mut tzmin = (bmin.z - orig.z) * inv_dz;
    let mut tzmax = (bmax.z - orig.z) * inv_dz;
    if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }
    if (tmin > tzmax) || (tzmin > tmax) { return None; }
    if tzmin > tmin { tmin = tzmin; }
    if tzmax < tmax { tmax = tzmax; }
    Some((tmin, tmax))
}

// Intersectar rayo con BVH y devolver la intersección más cercana (o vacía)
pub fn intersect_bvh(bvh: &BVH, objects: &[Cube], orig: &glm::Vec3, dir: &glm::Vec3) -> Intersect {
    fn traverse(node: &BVHNode, bvh: &BVH, objects: &[Cube], orig: &glm::Vec3, dir: &glm::Vec3, best: &mut Intersect) {
        match node {
            BVHNode::Leaf { bbox_min, bbox_max, start, count } => {
                if let Some((_t0, _t1)) = ray_intersect_aabb(orig, dir, bbox_min, bbox_max) {
                    for i in *start..(*start + *count) {
                        let obj_idx = bvh.indices[i];
                        let tmp = objects[obj_idx].ray_intersect(orig, dir);
                        if tmp.is_intersecting && tmp.distance < best.distance {
                            *best = tmp;
                        }
                    }
                }
            }
            BVHNode::Node { bbox_min, bbox_max, left, right } => {
                if let Some((_t0, _t1)) = ray_intersect_aabb(orig, dir, bbox_min, bbox_max) {
                    traverse(left, bvh, objects, orig, dir, best);
                    traverse(right, bvh, objects, orig, dir, best);
                }
            }
        }
    }
    let mut best = Intersect::empty();
    best.distance = f32::INFINITY;
    traverse(&bvh.root, bvh, objects, orig, dir, &mut best);
    best
}