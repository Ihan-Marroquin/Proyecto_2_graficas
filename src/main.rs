mod framebuffer;
mod ray_intersect;
mod cube;
mod materials;
mod texture;
mod bvh;

use framebuffer::Framebuffer;
use rayon::prelude::*;

use raylib::prelude::*;
use nalgebra_glm as glm;
use ray_intersect::RayIntersect;
use crate::materials::*;
use crate::cube::Cube;
use bvh::{BVH, build_bvh, intersect_bvh};
use std::sync::atomic::{AtomicUsize, Ordering};

// Diagnóstico y flags generales
static HIT_COUNT: AtomicUsize = AtomicUsize::new(0);
// Global exposure multiplier (tune to brighten/darken scene)
const EXPOSURE: f32 = 1.6;

// Construye el diorama de referencia.
#[allow(clippy::needless_range_loop)]
fn build_reference_diorama() -> Vec<Cube> {
    let mut v: Vec<Cube> = Vec::new();

    let mut grid: [[bool;20];20] = [[true;20];20];
    for row in 13..=20 {
        let r = (row - 1) as usize;
        for col in 1..=5 {
            let c = (col - 1) as usize;
            grid[r][c] = false;
        }
    }
    for row in 15..=20 {
        let r = (row - 1) as usize;
        let c = 6 - 1;
        grid[r][c] = false;
    }

    grid[16-1][9-1] = false;
    grid[15-1][9-1] = true; 

    let cube_size = 1.0_f32;
    for row in 0..20 {
        for col in 0..20 {
            if !grid[row][col] { continue; }
            // mapear celda a coordenadas del mundo
            let x = col as f32 * cube_size;
            let z = row as f32 * cube_size;
            let center = glm::vec3(x, 0.5 * cube_size, z);
            let mat = material_grass();
            // acentos en la cara superior para coordenadas específicas
            let user_positions = [(6,12),(6,13),(6,14),(7,16),(7,17)];
            let mut is_brown_top = false;
            for &(uc, ur) in user_positions.iter() {
                if (col + 1) == uc && (row + 1) == ur { is_brown_top = true; }
            }
            // Para celdas marcadas, usar top_material para pintar la cara superior
            let top_mat = if is_brown_top { Some(material_dirt_path()) } else { None };
            let cube = Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: mat, top_material: top_mat, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) };
            v.push(cube);
        }
    }
    // --- Capa 2 (y = 1.5) ---------------------------------
    let mut grid2: [[bool;20];20] = [[true;20];20];
    for row in 11..=20 { grid2[(row-1) as usize][0] = false; }
    for row in 12..=20 { grid2[(row-1) as usize][1] = false; }
    for row in 12..=20 { grid2[(row-1) as usize][2] = false; }
    for row in 13..=20 { grid2[(row-1) as usize][3] = false; }
    for row in 13..=20 { grid2[(row-1) as usize][4] = false; }
    for row in 14..=20 { grid2[(row-1) as usize][5] = false; }
    for row in 16..=20 { grid2[(row-1) as usize][6] = false; }
    for row in 19..=20 { grid2[(row-1) as usize][7] = false; }
    grid2[20-1][8] = false;

    grid2[16-1][9-1] = false;
    grid2[15-1][9-1] = true; 

    for row in 0..20 {
        for col in 0..20 {
            if !grid2[row][col] { continue; }
            let x = col as f32 * cube_size;
            let z = row as f32 * cube_size;
            let center = glm::vec3(x, 1.5 * cube_size, z);
            let mat = material_grass();
            let user_top_positions = [(6,12),(6,13),(7,13),(7,14),(7,15),(8,16)];
            let mut is_brown_top = false;
            for &(uc, ur) in user_top_positions.iter() {
                if (col + 1) == uc && (row + 1) == ur { is_brown_top = true; break; }
            }
            let top_mat = if is_brown_top { Some(material_dirt_path()) } else { None };
            let cube = Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: mat, top_material: top_mat, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) };
            v.push(cube);
        }
    }
    // --- Capa 3 (y = 2.5) -----------------------
    let mut grid3: [[bool;20];20] = [[true;20];20];
    for row in 10..=20 { grid3[(row-1) as usize][0] = false; }
    for row in 11..=20 { grid3[(row-1) as usize][1] = false; }
    for row in 11..=20 { grid3[(row-1) as usize][2] = false; }
    for row in 12..=20 { grid3[(row-1) as usize][3] = false; }
    for row in 12..=20 { grid3[(row-1) as usize][4] = false; }
    for row in 13..=20 { grid3[(row-1) as usize][5] = false; }
    for row in 13..=20 { grid3[(row-1) as usize][6] = false; }
    for row in 13..=20 { grid3[(row-1) as usize][7] = false; }
    for row in 13..=15 { grid3[(row-1) as usize][8] = false; }
    for row in 17..=20 { grid3[(row-1) as usize][8] = false; }
    grid3[13-1][9] = false; for row in 19..=20 { grid3[(row-1) as usize][9] = false; }
    grid3[13-1][10] = false; for row in 19..=20 { grid3[(row-1) as usize][10] = false; }
    grid3[13-1][11] = false; for row in 19..=20 { grid3[(row-1) as usize][11] = false; }
    grid3[13-1][12] = false; for row in 19..=20 { grid3[(row-1) as usize][12] = false; }
    grid3[13-1][13] = false; grid3[19-1][13] = false;
    for row in 13..=19 { grid3[(row-1) as usize][14] = false; }

    let mut diag_pillars: Vec<(i32,i32)> = Vec::new();
    let mut diag_stones: Vec<(i32,i32)> = Vec::new();
    let mut diag_dark: Vec<(i32,i32)> = Vec::new();

   
    grid3[16-1][9-1] = false;
    grid3[15-1][9-1] = false;

    for row in 0..20 {
        for col in 0..20 {
            if !grid3[row][col] { continue; }
            let x = col as f32 * cube_size;
            let z = row as f32 * cube_size;
            let center = glm::vec3(x, 2.5 * cube_size, z);
            let mut mat = material_grass();
            let mut top_mat: Option<crate::ray_intersect::Material> = None;

            let uc = (col + 1) as i32;
            let ur = (row + 1) as i32;

            let pillars = [ (10,14), (10,18), (14,14), (14,18) ];
            if pillars.iter().any(|&(x,y)| x == uc && y == ur) {
                mat = material_pillar();
                diag_pillars.push((uc, ur));
            }

            
            let stones = [
                (10,15),(10,17),(11,14),(12,14),(13,14),
                (14,15),(14,16),(14,17),
                (13,18),(12,18),(11,18)
            ];
            if stones.iter().any(|&(x,y)| x == uc && y == ur) {
                mat = material_stone();
                diag_stones.push((uc, ur));
            }

            let light_gray_positions = [
                (10,15),(11,14),(12,14),(13,14),(14,15),(14,16),(14,17),
                (13,18),(12,18),(11,18),(10,17)
            ];
            if light_gray_positions.iter().any(|&(x,y)| x == uc && y == ur) {
                mat = material_light_gray();
                diag_stones.push((uc, ur));
            }

            if uc == 10 && ur == 16 {
                mat = material_dark_wood();
                diag_dark.push((uc, ur));
            }
            if uc == 11 && (15..=17).contains(&ur) {
                mat = material_dark_wood();
                diag_dark.push((uc, ur));
            }
            if uc == 12 && (15..=17).contains(&ur) {
                mat = material_dark_wood();
                diag_dark.push((uc, ur));
            }
            if uc == 13 && (15..=17).contains(&ur) {
                mat = material_dark_wood();
                diag_dark.push((uc, ur));
            }

            let layer3_top_positions = [(6,12), (7,12)];
            for &(tc, tr) in layer3_top_positions.iter() {
                if tc == uc && tr == ur {
                    top_mat = Some(material_dirt_path());
                    break;
                }
            }

            let cube = Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: mat, top_material: top_mat.clone(), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) };
            v.push(cube);
        }
    }

    // --- Capa 4 (y = 3.5) --------------------
    let layer4_y = 3.5 * cube_size;

    let layer4_light_positions = [
        (10,15),(11,14),(12,14),(13,14),(14,15),(14,16),(14,17),
        (13,18),(12,18),(11,18),(10,17)
    ];
    let layer4_pillars = [ (10,14), (10,18), (14,14), (14,18) ];
    let layer4_dark_positions = [ (10,16) ];

    let mut grid4: [[bool;20];20] = [[true;20];20];
    for row in 10..=20 { grid4[(row-1) as usize][0] = false; } 
    for row in 10..=20 { grid4[(row-1) as usize][1] = false; } 
    for row in 11..=20 { grid4[(row-1) as usize][2] = false; } 
    for row in 11..=20 { grid4[(row-1) as usize][3] = false; }
    for row in 12..=20 { grid4[(row-1) as usize][4] = false; }
    for row in 12..=20 { grid4[(row-1) as usize][5] = false; }
    for row in 12..=20 { grid4[(row-1) as usize][6] = false; }
    for row in 12..=20 { grid4[(row-1) as usize][7] = false; }
    for row in 12..=20 { grid4[(row-1) as usize][8] = false; }
    for row in 12..=13 { grid4[(row-1) as usize][9] = false; }
    for row in 19..=20 { grid4[(row-1) as usize][9] = false; }
    for row in 12..=13 { grid4[(row-1) as usize][10] = false; }
    for row in 15..=17 { grid4[(row-1) as usize][10] = false; }
    for row in 19..=20 { grid4[(row-1) as usize][10] = false; }
    for row in 12..=13 { grid4[(row-1) as usize][11] = false; }
    for row in 15..=17 { grid4[(row-1) as usize][11] = false; }
    for row in 19..=20 { grid4[(row-1) as usize][11] = false; }
    for row in 12..=13 { grid4[(row-1) as usize][12] = false; }
    for row in 15..=17 { grid4[(row-1) as usize][12] = false; }
    for row in 19..=20 { grid4[(row-1) as usize][12] = false; }
    for row in 12..=13 { grid4[(row-1) as usize][13] = false; }
    for row in 19..=20 { grid4[(row-1) as usize][13] = false; }
    for row in 12..=20 { grid4[(row-1) as usize][14] = false; }

    for &(uc, ur) in layer4_light_positions.iter() { grid4[(ur-1) as usize][(uc-1) as usize] = false; }
    for &(uc, ur) in layer4_pillars.iter() { grid4[(ur-1) as usize][(uc-1) as usize] = false; }
    for &(uc, ur) in layer4_dark_positions.iter() { grid4[(ur-1) as usize][(uc-1) as usize] = false; }

    let mut diag_layer4_grass: Vec<(i32,i32)> = Vec::new();
    for row in 0..20 {
        for col in 0..20 {
            if !grid4[row][col] { continue; }
            let cx = col as f32 * cube_size;
            let cz = row as f32 * cube_size;
            let center = glm::vec3(cx, layer4_y, cz);
            let uc = (col + 1) as i32;
            let ur = (row + 1) as i32;
            let mut top_mat: Option<crate::ray_intersect::Material> = None;
            if (uc == 6 || uc == 7) && (9..=11).contains(&ur) {
                top_mat = Some(material_dirt_path());
            }
            v.push(Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_grass(), top_material: top_mat, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            diag_layer4_grass.push((uc, ur));
        }
    }

    let mut diag_layer4_light: Vec<(i32,i32)> = Vec::new();
    let mut diag_layer4_pillars: Vec<(i32,i32)> = Vec::new();
    let mut diag_layer4_dark: Vec<(i32,i32)> = Vec::new();

    for &(uc, ur) in layer4_light_positions.iter() {
        let cx = (uc - 1) as f32 * cube_size;
        let cz = (ur - 1) as f32 * cube_size;
        let center = glm::vec3(cx, layer4_y, cz);
        v.push(Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        diag_layer4_light.push((uc, ur));
    }

    for &(uc, ur) in layer4_pillars.iter() {
        let cx = (uc - 1) as f32 * cube_size;
        let cz = (ur - 1) as f32 * cube_size;
        let center = glm::vec3(cx, layer4_y, cz);
        v.push(Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_pillar(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        diag_layer4_pillars.push((uc, ur));
    }

    for &(uc, ur) in layer4_dark_positions.iter() {
        let cx = (uc - 1) as f32 * cube_size;
        let cz = (ur - 1) as f32 * cube_size;
        let center = glm::vec3(cx, layer4_y, cz);
        v.push(Cube { center, half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_dark_wood(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        diag_layer4_dark.push((uc, ur));
    }

    let stair_anchor = (9, 16);
    let (acol, arow) = ((stair_anchor.0 - 1) as usize, (stair_anchor.1 - 1) as usize);
    let x = acol as f32 * cube_size;
    let z = arow as f32 * cube_size;
    let slab_half_h = 0.25 * cube_size;
    let slab_center_y = 2.0 * cube_size;
    let mat = material_dark_wood();
    let center_a = glm::vec3(x, slab_center_y, z);
    let half_a = glm::vec3(0.5 * cube_size, slab_half_h, 0.5 * cube_size);
    v.push(Cube { center: center_a, half_size: half_a, rot_y: 0.0, material: mat.clone(), top_material: None, radius: glm::length(&half_a) });
    let center_b = glm::vec3(x + 0.5 * cube_size, slab_center_y, z);
    let half_b = glm::vec3(0.5 * cube_size, slab_half_h, 0.5 * cube_size);
    v.push(Cube { center: center_b, half_size: half_b, rot_y: 0.0, material: mat.clone(), top_material: None, radius: glm::length(&half_b) });
    let center_b_top = glm::vec3(x + 0.5 * cube_size, slab_center_y + (slab_half_h * 2.0), z);
    let half_b_top = glm::vec3(0.5 * cube_size, slab_half_h, 0.5 * cube_size);
    v.push(Cube { center: center_b_top, half_size: half_b_top, rot_y: 0.0, material: mat.clone(), top_material: None, radius: glm::length(&half_b_top) });
    let center = glm::vec3(9.5 * cube_size, 0.0, 9.5 * cube_size);

        v.retain(|obj| {
            if obj.half_size.y < 0.4 { return true; }
            if (obj.half_size.y > 0.45) && (obj.center.y - 2.5).abs() < 1e-3 && (obj.center.x - 8.0).abs() < 1e-3 && (obj.center.z - 15.0).abs() < 1e-3 {
                return false;
            }
            true
        });

    // --- Capa 5 (y = 4.5) -----------------------
        // Siempre usar la construcción determinista de la capa 5
            use materials::*;
            let pre_len = v.len();
            let cube_size = 1.0_f32;
            let layer5_y = 4.5 * cube_size;

            let mut grid5: [[bool;20];20] = [[true;20];20];

            for row in 7..=20 { grid5[(row-1) as usize][0] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][1] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][2] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][3] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][4] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][5] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][6] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][7] = false; }
            for row in 9..=20 { grid5[(row-1) as usize][8] = false; }
            for row in 11..=13 { grid5[(row-1) as usize][9] = false; }
            for row in 19..=20 { grid5[(row-1) as usize][9] = false; }
            for row in 11..=13 { grid5[(row-1) as usize][10] = false; }
            for row in 15..=17 { grid5[(row-1) as usize][10] = false; }
            for row in 19..=20 { grid5[(row-1) as usize][10] = false; }
            for row in 11..=13 { grid5[(row-1) as usize][11] = false; }
            for row in 15..=17 { grid5[(row-1) as usize][11] = false; }
            for row in 19..=20 { grid5[(row-1) as usize][11] = false; }
            for row in 11..=13 { grid5[(row-1) as usize][12] = false; }
            for row in 15..=17 { grid5[(row-1) as usize][12] = false; }
            for row in 19..=20 { grid5[(row-1) as usize][12] = false; }
            for row in 11..=13 { grid5[(row-1) as usize][13] = false; }
            for row in 19..=20 { grid5[(row-1) as usize][13] = false; }
            for row in 11..=20 { grid5[(row-1) as usize][14] = false; }
            for row in 19..=20 { grid5[(row-1) as usize][15] = false; }
            grid5[20-1][16] = false;

            let pillars = [(10,14),(10,18),(14,14),(14,18)];
            for &(pc, pr) in pillars.iter() { grid5[(pr-1) as usize][(pc-1) as usize] = false; }
            grid5[(16-1) as usize][(10-1) as usize] = false;
            let gray_positions = [(10,15),(11,14),(13,14),(13,18),(11,18),(10,17)];
            for &(gc, gr) in gray_positions.iter() { grid5[(gr-1) as usize][(gc-1) as usize] = false; }
            for row in 15..=17 { grid5[(row-1) as usize][(14-1) as usize] = false; } // (14,15)-(14,17)
            grid5[(14-1) as usize][(12-1) as usize] = false;
            grid5[(18-1) as usize][(12-1) as usize] = false;
            grid5[(10-1) as usize][(10-1) as usize] = false;
            let water_positions = vec![
                (2,2),(2,3),(2,4),
                (3,3),(3,4),(3,5),(3,6),
                (4,2),(4,3),(4,4),(4,5),
                (5,2),(5,3),(5,4),(5,5),(5,6),
                (6,3),(6,4),(6,5),
                (7,3),(7,4),(7,5),(7,6),(7,7),
                (8,2),(8,3),(8,4),(8,5),(8,6),(8,7),
                (9,2),(9,3),(9,4),(9,5),(9,6),(9,7),
                (10,3),(10,4),(10,5),(10,6),(10,7),
                (11,3),(11,4),(11,5),
                (12,5),(12,6),(12,7),(12,8),
                (13,5),(13,6),(13,7),(13,8),
            ];
            for &(wc, wr) in water_positions.iter() { grid5[(wr-1) as usize][(wc-1) as usize] = false; }

            for row in 0..20 {
                for col in 0..20 {
                    if !grid5[row][col] { continue; }
                    let cx = col as f32 * cube_size;
                    let cz = row as f32 * cube_size;
                    let uc = (col + 1) as i32;
                    let ur = (row + 1) as i32;
                    let top_mat = if (uc == 6 && (ur == 6 || ur == 7 || ur == 8)) || (uc == 7 && ur == 8) {
                        Some(material_dirt_path())
                    } else {
                        None
                    };
                    v.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_grass(), top_material: top_mat, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                }
            }

            for &(pc, pr) in pillars.iter() {
                let cx = (pc-1) as f32 * cube_size;
                let cz = (pr-1) as f32 * cube_size;
                v.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_pillar(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
            v.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (16-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_dark_wood(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            for &(gc, gr) in gray_positions.iter() {
                v.push(Cube { center: glm::vec3((gc-1) as f32 * cube_size, layer5_y, (gr-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
            for row in 15..=18 { v.push(Cube { center: glm::vec3((14-1) as f32 * cube_size, layer5_y, (row-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
            v.push(Cube { center: glm::vec3((12-1) as f32 * cube_size, layer5_y, (14-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_glass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            v.push(Cube { center: glm::vec3((12-1) as f32 * cube_size, layer5_y, (18-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_glass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            v.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (10-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            for &(wc, wr) in water_positions.iter() {
                let cx = (wc-1) as f32 * cube_size;
                let cz = (wr-1) as f32 * cube_size;
                v.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_water(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }

            let water_cells: Vec<(i32,i32)> = water_positions.iter().map(|&(a,b)| (a,b)).collect();
            for (wc, wr) in water_cells.iter() {
                for i in 1..=3 {
                    let y = layer5_y - (i as f32) * 0.85;
                    v.push(make_small_cube(*wc, *wr, y, material_water()));
                }
                v.push(make_small_droplet(*wc, *wr, layer5_y - 0.45, material_water()));
            }

            let reserved = vec![
                (10i32,14i32),(10,18),(14,14),(14,18),
                (10,16),
                (10,15),(11,14),(13,14),(14,15),(14,16),(14,17),(13,18),(12,18),(11,18),(10,17),(14,18),
                (12,14),(12,18),
                (10,10),
                (2,2),(2,3),(2,4),
                (3,3),(3,4),(3,5),(3,6),
                (4,2),(4,3),(4,4),(4,5),
                (5,2),(5,3),(5,4),(5,5),(5,6),
                (6,3),(6,4),(6,5),
                (7,3),(7,4),(7,5),(7,6),(7,7),
                (8,2),(8,3),(8,4),(8,5),(8,6),(8,7),
                (9,2),(9,3),(9,4),(9,5),(9,6),(9,7),
                (10,3),(10,4),(10,5),(10,6),(10,7),
                (11,3),(11,4),(11,5),
                (12,5),(12,6),(12,7),(12,8),
                (13,5),(13,6),(13,7),(13,8)
            ];
            v.retain(|obj| {
                if (obj.center.y - layer5_y).abs() < 1e-3 {
                    let gc = (obj.center.x / cube_size).round() as i32 + 1;
                    let gr = (obj.center.z / cube_size).round() as i32 + 1;
                    let d = obj.material.diffuse;
                    if d.r == 80 && d.g == 180 && d.b == 70 && reserved.iter().any(|&(c,r)| c==gc && r==gr) {
                        return false;
                    }
                }
                true
            });

            let _added = v.len() - pre_len;

            // --- Capa 6 (y = 5.5) -----------------------
            // Construcción determinista de la capa 6 (inmediatamente arriba de la capa 5)
            {
                let cube_size = 1.0_f32;
                let layer6_y = 5.5 * cube_size;
                let mut grid6: [[bool;20];20] = [[true;20];20];

                // Marcar celdas que NO existen según especificación
                for row in 3..=20 { grid6[(row-1) as usize][0] = false; }
                for row in 2..=20 { grid6[(row-1) as usize][1] = false; }
                for row in 3..=20 { grid6[(row-1) as usize][2] = false; }
                for row in 2..=20 { grid6[(row-1) as usize][3] = false; }
                for row in 2..=20 { grid6[(row-1) as usize][4] = false; } 
                for row in 3..=20 { grid6[(row-1) as usize][5] = false; }
                for row in 3..=20 { grid6[(row-1) as usize][6] = false; }
                for row in 2..=20 { grid6[(row-1) as usize][7] = false; } 
                for row in 2..=20 { grid6[(row-1) as usize][8] = false; } 
                for row in 3..=13 { grid6[(row-1) as usize][9] = false; } 
                for row in 19..=20 { grid6[(row-1) as usize][9] = false; } 
                for row in 3..=7  { grid6[(row-1) as usize][10] = false; }
                for row in 10..=13 { grid6[(row-1) as usize][10] = false; }
                for row in 15..=17 { grid6[(row-1) as usize][10] = false; }
                for row in 19..=20 { grid6[(row-1) as usize][10] = false; }
                for row in 4..=13 { grid6[(row-1) as usize][11] = false; }
                for row in 15..=17 { grid6[(row-1) as usize][11] = false; }
                for row in 19..=20 { grid6[(row-1) as usize][11] = false; }
                for row in 4..=13 { grid6[(row-1) as usize][12] = false; }
                for row in 15..=17 { grid6[(row-1) as usize][12] = false; } 
                for row in 19..=20 { grid6[(row-1) as usize][12] = false; }
                for row in 10..=13 { grid6[(row-1) as usize][13] = false; }
                for row in 19..=20 { grid6[(row-1) as usize][13] = false; }
                for row in 11..=20 { grid6[(row-1) as usize][14] = false; }
                for row in 18..=20 { grid6[(row-1) as usize][15] = false; }
                for row in 19..=20 { grid6[(row-1) as usize][16] = false; }
                for row in 19..=20 { grid6[(row-1) as usize][17] = false; }
                for row in 19..=20 { grid6[(row-1) as usize][18] = false; } 
                grid6[(20-1) as usize][19] = false;

                // Posiciones especiales que SÍ existen
                let pumpkins = [(11,8),(11,9)];
                let pillars = [(10,14),(10,18),(14,14),(14,18)];
                let gray_positions = [ (10,15),(10,16),(10,17), (14,15),(14,16),(14,17), (11,14),(12,14),(13,14), (11,18),(12,18),(13,18) ];

                for row in 0..20 {
                    for col in 0..20 {
                        if !grid6[row][col] { continue; }
                        let cx = col as f32 * cube_size;
                        let cz = row as f32 * cube_size;
                        let uc = (col + 1) as i32;
                        let ur = (row + 1) as i32;
                        let is_pumpkin = pumpkins.iter().any(|&(c,r)| c==uc && r==ur);
                        let mat = if is_pumpkin {
                            material_pumpkin_side()
                        } else if pillars.iter().any(|&(c,r)| c==uc && r==ur) {
                            material_pillar()
                        } else if gray_positions.iter().any(|&(c,r)| c==uc && r==ur) {
                            material_light_gray()
                        } else {
                            material_grass()
                        };
                        let top_mat = if is_pumpkin { Some(material_pumpkin_top()) } else { None };
                        v.push(Cube { center: glm::vec3(cx, layer6_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: mat, top_material: top_mat, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                    }
                }
            }

    let cam_x = -12.5_f32;
    let cam_z = -10.5_f32;

    let target = glm::vec3(0.0_f32 * cube_size, 0.5 * cube_size, 19.0_f32 * cube_size);
    let rel = target - center;

    let mut best_angle: f32 = 0.0;
    let mut best_dist = f32::INFINITY;
    let _two_pi = std::f32::consts::PI * 2.0;
    for deg in 0..360 {
        let theta = (deg as f32) * (std::f32::consts::PI / 180.0);
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let rx = rel.x * cos_t - rel.z * sin_t;
        let rz = rel.x * sin_t + rel.z * cos_t;
        let rotated = glm::vec3(rx + center.x, target.y, rz + center.z);
        let dx = rotated.x - cam_x;
        let dz = rotated.z - cam_z;
        let dist2 = dx * dx + dz * dz;
        if dist2 < best_dist { best_dist = dist2; best_angle = theta; }
    }

    // --- Capa 7 (y = 6.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer7_y = 6.5 * cube_size;
        let mut grid7: [[bool;20];20] = [[true;20];20];

        for row in 2..=20 { grid7[(row-1) as usize][0] = false; } // (1,2)-(1,20)
        for row in 1..=20 { grid7[(row-1) as usize][1usize] = false; } // columna 2
        for row in 1..=20 { grid7[(row-1) as usize][2usize] = false; } // columna 3
        for row in 2..=20 { grid7[(row-1) as usize][3] = false; } // (4,2)-(4,20)
        for row in 2..=20 { grid7[(row-1) as usize][4] = false; } // (5,2)-(5,20)
        for row in 1..=20 { for col in 6..=9 { grid7[(row-1) as usize][(col-1) as usize] = false; } }
        for row in 1..=13 { grid7[(row-1) as usize][9] = false; } // (10,1)-(10,13)
        for row in 19..=20 { grid7[(row-1) as usize][9] = false; } // (10,19)-(10,20)
        for row in 1..=13 { grid7[(row-1) as usize][10] = false; } // (11,1)-(11,13)
        for row in 15..=17 { grid7[(row-1) as usize][10] = false; } // (11,15)-(11,17)
        for row in 19..=20 { grid7[(row-1) as usize][10] = false; } // (11,19)-(11,20)
        for row in 1..=13 { grid7[(row-1) as usize][11] = false; } // (12,1)-(12,13)
        for row in 15..=17 { grid7[(row-1) as usize][11] = false; } // (12,15)-(12,17)
        for row in 19..=20 { grid7[(row-1) as usize][11] = false; } // (12,19)-(12,20)
        for row in 4..=13 { grid7[(row-1) as usize][12] = false; } // (13,4)-(13,13)
        for row in 15..=17 { grid7[(row-1) as usize][12] = false; } // (13,15)-(13,17)
        for row in 19..=20 { grid7[(row-1) as usize][12] = false; } // (13,19)-(13,20)
        for row in 10..=13 { grid7[(row-1) as usize][13] = false; } // (14,10)-(14,13)
        for row in 19..=20 { grid7[(row-1) as usize][13] = false; } // (14,19)-(14,20)
        for row in 11..=20 { grid7[(row-1) as usize][14] = false; } // (15,11)-(15,20)
        for row in 12..=20 { grid7[(row-1) as usize][15] = false; } // (16,12)-(16,20)
        for row in 18..=20 { grid7[(row-1) as usize][16] = false; } // (17,18)-(17,20)
        for row in 19..=20 { grid7[(row-1) as usize][17] = false; } // (18,19)-(18,20)
        for row in 19..=20 { grid7[(row-1) as usize][18] = false; } // (19,19)-(19,20)
        for row in 19..=20 { grid7[(row-1) as usize][19] = false; } // (20,19)-(20,20)

        // Posiciones especiales que SÍ existen
        // Pilares puntuales históricos
        let pillars = [(10,14),(10,18),(14,14),(14,18)];
        let pillar_ranges: &[(i32,i32,i32,i32)] = &[
            (9,13,9,19),
            (10,13,10,19),
            (11,13,11,19),
            (12,13,12,19),
            (13,13,13,19),
            (14,13,14,19),
            (15,13,15,19),

        ];
        // Forzar que las celdas en estos rangos existan para poder convertirlas en pilares
        for &(c1,r1,c2,r2) in pillar_ranges.iter() {
            for cc in c1..=c2 {
                for rr in r1..=r2 {
                    let ci = (cc - 1) as usize;
                    let ri = (rr - 1) as usize;
                    if ci < 20 && ri < 20 {
                        grid7[ri][ci] = true;
                    }
                }
            }
        }
        let gray_positions = [ (10,15),(10,16),(10,17), (14,15),(14,16),(14,17), (11,14),(12,14),(13,14), (11,18),(12,18),(13,18) ];

        for row in 0..20 {
            for col in 0..20 {
                if !grid7[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                let is_pillar = pillars.iter().any(|&(c,r)| c==uc && r==ur)
                    || pillar_ranges.iter().any(|&(c1,r1,c2,r2)| uc >= c1 && uc <= c2 && ur >= r1 && ur <= r2);
                let mat = if is_pillar {
                    material_pillar()
                } else if gray_positions.iter().any(|&(c,r)| c==uc && r==ur) {
                    material_light_gray()
                } else {
                    material_grass()
                };
                v.push(Cube { center: glm::vec3(cx, layer7_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
        }
    }

    // --- Capa 8 (y = 7.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer8_y = 7.5 * cube_size;
        let mut grid8: [[bool;20];20] = [[true;20];20];

        // Marcar celdas que NO existen según especificación
        for col in 1..=9 {
            for row in 1..=20 {
                grid8[(row-1) as usize][(col-1) as usize] = false;
            }
        }
        for row in 1..=13 { grid8[(row-1) as usize][9] = false; } // (10,1)-(10,13)
        for row in 19..=20 { grid8[(row-1) as usize][9] = false; } // (10,19)-(10,20)
        for row in 1..=13 { grid8[(row-1) as usize][10] = false; } // (11,1)-(11,13)
        for row in 19..=20 { grid8[(row-1) as usize][10] = false; } // (11,19)-(11,20)
        for row in 1..=13 { grid8[(row-1) as usize][11] = false; } // (12,1)-(12,13)
        for row in 19..=20 { grid8[(row-1) as usize][11] = false; } // (12,19)-(12,20)
        for row in 4..=13 { grid8[(row-1) as usize][12] = false; } // (13,4)-(13,13)
        for row in 19..=20 { grid8[(row-1) as usize][12] = false; } // (13,19)-(13,20)
        for row in 8..=13 { grid8[(row-1) as usize][13] = false; } // (14,8)-(14,13)
        for row in 19..=20 { grid8[(row-1) as usize][13] = false; } // (14,19)-(14,20)
        for row in 10..=20 { grid8[(row-1) as usize][14] = false; } // (15,10)-(15,20)
        for row in 11..=20 { grid8[(row-1) as usize][15] = false; } // (16,11)-(16,20)
        for row in 17..=20 { grid8[(row-1) as usize][16] = false; } // (17,17)-(17,20)
        for row in 18..=20 { grid8[(row-1) as usize][17] = false; } // (18,18)-(18,20)
        for row in 18..=20 { grid8[(row-1) as usize][18] = false; } // (19,18)-(19,20)
        for row in 18..=20 { grid8[(row-1) as usize][19] = false; } // (20,18)-(20,20)

        // Pilares en (10..14 columns) filas 14..18
        let pillar_ranges8: &[(i32,i32,i32,i32)] = &[
            (10,14,10,18),
            (11,14,11,18),
            (12,14,12,18),
            (13,14,13,18),
            (14,14,14,18),
        ];
        // Forzar que las celdas en los rangos de pilar existan
        for &(c1,r1,c2,r2) in pillar_ranges8.iter() {
            for cc in c1..=c2 {
                for rr in r1..=r2 {
                    let ci = (cc - 1) as usize;
                    let ri = (rr - 1) as usize;
                    if ci < 20 && ri < 20 {
                        grid8[ri][ci] = true;
                    }
                }
            }
        }

        for row in 0..20 {
            for col in 0..20 {
                if !grid8[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                let is_pillar = pillar_ranges8.iter().any(|&(c1,r1,c2,r2)| uc >= c1 && uc <= c2 && ur >= r1 && ur <= r2);
                let mat = if is_pillar { material_pillar() } else { material_grass() };
                v.push(Cube { center: glm::vec3(cx, layer8_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
        }
    }

    // --- Capa 9 (y = 8.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer9_y = 8.5 * cube_size;
        let mut grid9: [[bool;20];20] = [[true;20];20];

        // Excluir columnas 1..=10 completas
        for col in 1..=10 {
            for row in 1..=20 {
                grid9[(row-1) as usize][(col-1) as usize] = false;
            }
        }

        // Columnas/filas específicas según tu lista
        for row in 1..=14 { grid9[(row-1) as usize][10] = false; } // (11,1)-(11,14)
        for row in 18..=20 { grid9[(row-1) as usize][10] = false; } // (11,18)-(11,20)
        for row in 1..=14 { grid9[(row-1) as usize][11] = false; } // (12,1)-(12,14)
        for row in 18..=20 { grid9[(row-1) as usize][11] = false; } // (12,18)-(12,20)
        for row in 4..=14 { grid9[(row-1) as usize][12] = false; } // (13,4)-(13,14)
        for row in 18..=20 { grid9[(row-1) as usize][12] = false; } // (13,18)-(13,20)

        for row in 8..=20 { grid9[(row-1) as usize][13] = false; }
        for row in 8..=20 { grid9[(row-1) as usize][14] = false; }
        for row in 9..=20 { grid9[(row-1) as usize][15] = false; }
        for row in 15..=20 { grid9[(row-1) as usize][16] = false; }
        for row in 15..=20 { grid9[(row-1) as usize][17] = false; }
        for row in 16..=20 { grid9[(row-1) as usize][18] = false; }
        for row in 17..=20 { grid9[(row-1) as usize][19] = false; }

        // Pilar ranges: (11,15)-(11,17), (12,15)-(12,17), (13,15)-(13,17)
        let pillar_ranges9: &[(i32,i32,i32,i32)] = &[
            (11,15,11,17),
            (12,15,12,17),
            (13,15,13,17),
        ];
        for &(c1,r1,c2,r2) in pillar_ranges9.iter() {
            for cc in c1..=c2 {
                for rr in r1..=r2 {
                    let ci = (cc - 1) as usize;
                    let ri = (rr - 1) as usize;
                    if ci < 20 && ri < 20 {
                        grid9[ri][ci] = true;
                    }
                }
            }
        }

        for row in 0..20 {
            for col in 0..20 {
                if !grid9[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                let is_pillar = pillar_ranges9.iter().any(|&(c1,r1,c2,r2)| uc >= c1 && uc <= c2 && ur >= r1 && ur <= r2);
                let mat = if is_pillar { material_pillar() } else { material_grass() };
                v.push(Cube { center: glm::vec3(cx, layer9_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
        }
    }

    // --- Capa 10 (y = 9.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer10_y = 9.5 * cube_size;
        let mut grid10: [[bool;20];20] = [[true;20];20];

        // Excluir columnas 1..=12 completas
        for col in 1..=12 {
            for row in 1..=20 {
                grid10[(row-1) as usize][(col-1) as usize] = false;
            }
        }

        for row in 3..=20 { grid10[(row-1) as usize][12] = false; }
        for row in 8..=20 { grid10[(row-1) as usize][13] = false; }
        for row in 8..=20 { grid10[(row-1) as usize][14] = false; }
        for row in 8..=20 { grid10[(row-1) as usize][15] = false; }
        for row in 14..=20 { grid10[(row-1) as usize][16] = false; }
        for row in 15..=20 { grid10[(row-1) as usize][17] = false; }
        for row in 16..=20 { grid10[(row-1) as usize][18] = false; }
        for row in 16..=20 { grid10[(row-1) as usize][19] = false; }

        for row in 0..20 {
            for col in 0..20 {
                if !grid10[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                v.push(Cube { center: glm::vec3(cx, layer10_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
        }
    }

    // --- Capa 11 (y = 10.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer11_y = 10.5 * cube_size;
        let mut grid11: [[bool;20];20] = [[true;20];20];

        // Excluir columnas 1..=13 completas
        for col in 1..=13 {
            for row in 1..=20 {
                grid11[(row-1) as usize][(col-1) as usize] = false;
            }
        }

        for row in 5..=6 { grid11[(row-1) as usize][(14-1) as usize] = false; }
        for row in 8..=20 { grid11[(row-1) as usize][(14-1) as usize] = false; }
        for row in 8..=20 { grid11[(row-1) as usize][(15-1) as usize] = false; }
        for row in 8..=20 { grid11[(row-1) as usize][(16-1) as usize] = false; }
        for row in 13..=20 { grid11[(row-1) as usize][(17-1) as usize] = false; }
        for row in 14..=20 { grid11[(row-1) as usize][(18-1) as usize] = false; }
        for row in 15..=20 { grid11[(row-1) as usize][(19-1) as usize] = false; }
        for row in 15..=20 { grid11[(row-1) as usize][(20-1) as usize] = false; }

        // Coloca cubos: calabaza en (14,7) si la celda existe, el resto pasto
        for row in 0..20 {
            for col in 0..20 {
                if !grid11[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                if uc == 14 && ur == 7 {
                    v.push(Cube { center: glm::vec3(cx, layer11_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                } else {
                    v.push(Cube { center: glm::vec3(cx, layer11_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                }
            }
        }
    }

    // --- Capa 12 (y = 11.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer12_y = 11.5 * cube_size;
        let mut grid12: [[bool;20];20] = [[true;20];20];

        // Excluir columnas 1..=13 completas
        for col in 1..=13 {
            for row in 1..=20 {
                grid12[(row-1) as usize][(col-1) as usize] = false;
            }
        }

        for row in 2..=20 { grid12[(row-1) as usize][(14-1) as usize] = false; }
        for row in 5..=20 { grid12[(row-1) as usize][(15-1) as usize] = false; }
        grid12[(6-1) as usize][(16-1) as usize] = false;
        for row in 8..=20 { grid12[(row-1) as usize][(16-1) as usize] = false; }
        for row in 12..=20 { grid12[(row-1) as usize][(17-1) as usize] = false; }
        for row in 12..=20 { grid12[(row-1) as usize][(18-1) as usize] = false; }
        for row in 12..=20 { grid12[(row-1) as usize][(19-1) as usize] = false; }
        for row in 13..=20 { grid12[(row-1) as usize][(20-1) as usize] = false; }

        // Calabazas: (15,4), (16,7)
        for row in 0..20 {
            for col in 0..20 {
                if !grid12[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                if (uc == 15 && ur == 4) || (uc == 16 && ur == 7) {
                    v.push(Cube { center: glm::vec3(cx, layer12_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                } else {
                    v.push(Cube { center: glm::vec3(cx, layer12_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                }
            }
        }
    }

    // --- Capa 13 (y = 12.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer13_y = 12.5 * cube_size;
        let mut grid13: [[bool;20];20] = [[true;20];20];

        // Excluir columnas 1..=15 completas
        for col in 1..=15 {
            for row in 1..=20 {
                grid13[(row-1) as usize][(col-1) as usize] = false;
            }
        }

        grid13[(1-1) as usize][(16-1) as usize] = false;
        for row in 3..=20 { grid13[(row-1) as usize][(16-1) as usize] = false; }
        for row in 9..=20 { grid13[(row-1) as usize][(17-1) as usize] = false; }
        for row in 11..=20 { grid13[(row-1) as usize][(18-1) as usize] = false; }
        for row in 11..=20 { grid13[(row-1) as usize][(19-1) as usize] = false; }
        for row in 12..=20 { grid13[(row-1) as usize][(20-1) as usize] = false; }

        // Calabaza en (16,2)
        for row in 0..20 {
            for col in 0..20 {
                if !grid13[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                if uc == 16 && ur == 2 {
                    v.push(Cube { center: glm::vec3(cx, layer13_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                } else {
                    v.push(Cube { center: glm::vec3(cx, layer13_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                }
            }
        }
    }

    // --- Capa 14 (y = 13.5) -----------------------
    {
        let cube_size = 1.0_f32;
        let layer14_y = 13.5 * cube_size;
        let mut grid14: [[bool;20];20] = [[true;20];20];

        // Excluir columnas 1..=17 completas
        for col in 1..=17 {
            for row in 1..=20 {
                grid14[(row-1) as usize][(col-1) as usize] = false;
            }
        }

        for row in 1..=6 { grid14[(row-1) as usize][(18-1) as usize] = false; }
        for row in 8..=20 { grid14[(row-1) as usize][(18-1) as usize] = false; }
        grid14[(1-1) as usize][(19-1) as usize] = false;
        for row in 9..=20 { grid14[(row-1) as usize][(19-1) as usize] = false; }
        grid14[(1-1) as usize][(20-1) as usize] = false;
        for row in 4..=5 { grid14[(row-1) as usize][(20-1) as usize] = false; }
        for row in 9..=20 { grid14[(row-1) as usize][(20-1) as usize] = false; }

        // Calabaza: (19,4)
        for row in 0..20 {
            for col in 0..20 {
                if !grid14[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                if uc == 19 && ur == 4 {
                    v.push(Cube { center: glm::vec3(cx, layer14_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                } else {
                    v.push(Cube { center: glm::vec3(cx, layer14_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                }
            }
        }
    }

   
    {
        let dirt_r = 120u8; let dirt_g = 85u8; let dirt_b = 55u8;
        let mut dirt_positions: Vec<(f32,f32,f32)> = Vec::new();
        let mut kept: Vec<Cube> = Vec::new();
        for obj in v.into_iter() {
            let c = obj.material.diffuse;
            if c.r == dirt_r && c.g == dirt_g && c.b == dirt_b {
                dirt_positions.push((obj.center.x, obj.center.z, obj.center.y));
            } else {
                kept.push(obj);
            }
        }
        v = kept;
        for (dx, dz, dy) in dirt_positions.iter() {
            let target_y = *dy - 1.0;
            if let Some(target) = v.iter_mut().find(|c| (c.center.x - *dx).abs() < 1e-3 && (c.center.z - *dz).abs() < 1e-3 && (c.center.y - target_y).abs() < 1e-2) {
                target.top_material = Some(material_dirt_path());
            } else {
                if let Some(target2) = v.iter_mut().filter(|c| (c.center.x - *dx).abs() < 1e-3 && (c.center.z - *dz).abs() < 1e-3 && c.center.y < *dy).max_by(|a,b| a.center.y.partial_cmp(&b.center.y).unwrap_or(std::cmp::Ordering::Equal)) {
                    target2.top_material = Some(material_dirt_path());
                }
            }
        }
    }

    let cos_b = best_angle.cos();
    let sin_b = best_angle.sin();
    for obj in v.iter_mut() {
        let p = obj.center - center;
        let nx = p.x * cos_b - p.z * sin_b;
        let nz = p.x * sin_b + p.z * cos_b;
        obj.center = glm::vec3(nx + center.x, obj.center.y, nz + center.z);
    }
   
    {
        let cube_size = 1.0_f32;
        let layer1_y = 0.5 * cube_size;
        let layer2_y = 1.5 * cube_size;
        let layer1_positions = [(6i32,14i32),(7,16),(7,17)];
        let layer2_positions = [(6i32,13i32),(7,13),(7,14),(7,15),(8,16)];
        for &(c,r) in layer1_positions.iter() {
            let gx = (c - 1) as f32 * cube_size;
            let gz = (r - 1) as f32 * cube_size;
            let px = gx - center.x;
            let pz = gz - center.z;
            let rx = px * cos_b - pz * sin_b + center.x;
            let rz = px * sin_b + pz * cos_b + center.z;
            if let Some(obj) = v.iter_mut().find(|o| (o.center.x - rx).abs() < 1e-3 && (o.center.z - rz).abs() < 1e-3 && (o.center.y - layer1_y).abs() < 1e-3) {
                obj.top_material = Some(material_dirt_path());
            }
        }
        for &(c,r) in layer2_positions.iter() {
            let gx = (c - 1) as f32 * cube_size;
            let gz = (r - 1) as f32 * cube_size;
            let px = gx - center.x;
            let pz = gz - center.z;
            let rx = px * cos_b - pz * sin_b + center.x;
            let rz = px * sin_b + pz * cos_b + center.z;
            if let Some(obj) = v.iter_mut().find(|o| (o.center.x - rx).abs() < 1e-3 && (o.center.z - rz).abs() < 1e-3 && (o.center.y - layer2_y).abs() < 1e-3) {
                obj.top_material = Some(material_dirt_path());
            }
        }
    }
    {
        let cube_size = 1.0_f32;
        let checks = vec![ (1.0, 0.5, vec![(6,14),(7,16),(7,17)]), (1.0, 1.5, vec![(6,13),(7,13),(7,14),(7,15),(8,16)]) ];
        for (_scale, y, list) in checks.into_iter() {
            for (c,r) in list.into_iter() {
                let gx = (c - 1) as f32 * cube_size;
                let gz = (r - 1) as f32 * cube_size;
                let px = gx - center.x;
                let pz = gz - center.z;
                let rx = px * cos_b - pz * sin_b + center.x;
                let rz = px * sin_b + pz * cos_b + center.z;
                if let Some(obj) = v.iter().find(|o| (o.center.x - rx).abs() < 1e-3 && (o.center.z - rz).abs() < 1e-3 && (o.center.y - y).abs() < 1e-3) {
                    match &obj.top_material {
                        Some(m) => println!("diag: found obj at ({},{}) y={} top_material.texture={:?}", c, r, y, m.texture),
                        None => println!("diag: found obj at ({},{}) y={} but top_material=None", c, r, y),
                    }
                } else {
                    println!("diag: no object at ({},{}) y={}", c, r, y);
                }
            }
        }
    }
    {
        let cube_size = 1.0_f32;
        let layer5_y = 4.5 * cube_size;
        let reserved = [(10i32,14i32),(10,18),(14,14),(14,18),(10,16),
            (10,15),(11,14),(13,14),(14,15),(14,16),(14,17),(13,18),(12,18),(11,18),(10,17),
            (12,14),(12,18),(10,10),(2,2),(2,3),(2,4)];
        let mut rotated_reserved: Vec<glm::Vec3> = Vec::new();
        for &(gc, gr) in reserved.iter() {
            let px = (gc - 1) as f32 * cube_size;
            let pz = (gr - 1) as f32 * cube_size;
            let p = glm::vec3(px, layer5_y, pz) - center;
            let rx = p.x * cos_b - p.z * sin_b;
            let rz = p.x * sin_b + p.z * cos_b;
            rotated_reserved.push(glm::vec3(rx + center.x, layer5_y, rz + center.z));
        }
        v.retain(|obj| {
            if (obj.center.y - layer5_y).abs() < 1e-3 {
                let d = obj.material.diffuse;
                if d.r == 80 && d.g == 180 && d.b == 70 {
                    for rr in rotated_reserved.iter() {
                        let dx = obj.center.x - rr.x;
                        let dz = obj.center.z - rr.z;
                        if (dx*dx + dz*dz) < 0.25 { 
                            return false;
                        }
                    }
                }
            }
            true
        });
    }
    v
}

#[allow(clippy::needless_range_loop)]
fn build_reference_diorama_layers() -> Vec<(String, Vec<Cube>)> {
    let all = build_reference_diorama();
    let mut layer1: Vec<Cube> = Vec::new();
    let mut layer2: Vec<Cube> = Vec::new();
    let mut layer3: Vec<Cube> = Vec::new();
    let mut layer4: Vec<Cube> = Vec::new();
    let mut layer5: Vec<Cube> = Vec::new();
    let mut layer6: Vec<Cube> = Vec::new();
    let mut layer7: Vec<Cube> = Vec::new();
    let mut layer8: Vec<Cube> = Vec::new();
    let mut layer9: Vec<Cube> = Vec::new();
    let mut layer10: Vec<Cube> = Vec::new();
    let mut layer11: Vec<Cube> = Vec::new();
    let mut layer12: Vec<Cube> = Vec::new();
    let mut layer13: Vec<Cube> = Vec::new();
    let mut layer14: Vec<Cube> = Vec::new();

    for obj in all.into_iter() {
        let y = obj.center.y;
        if (y - 0.5).abs() < 0.4 {
            layer1.push(obj);
        } else if (y - 1.5).abs() < 0.4 {
            layer2.push(obj);
        } else if (y - 2.5).abs() < 0.4 {
            layer3.push(obj);
        } else if (y - 3.5).abs() < 0.4 {
            layer4.push(obj);
        } else if (y - 4.5).abs() < 0.4 {
            layer5.push(obj);
        } else if (y - 5.5).abs() < 0.4 {
            layer6.push(obj);
        } else if (y - 6.5).abs() < 0.4 {
            layer7.push(obj);
        } else if (y - 7.5).abs() < 0.4 {
            layer8.push(obj);
        } else if (y - 8.5).abs() < 0.4 {
            layer9.push(obj);
        } else if (y - 9.5).abs() < 0.4 {
            layer10.push(obj);
        } else if (y - 10.5).abs() < 0.4 {
            layer11.push(obj);
        } else if (y - 11.5).abs() < 0.4 {
            layer12.push(obj);
        } else if (y - 12.5).abs() < 0.4 {
            layer13.push(obj);
        } else if (y - 13.5).abs() < 0.4 {
            layer14.push(obj);
        } else {
            layer5.push(obj);
        }
    }

    {
        let pre_len = layer5.len();
        let cube_size = 1.0_f32;
        let layer5_y = 4.5 * cube_size;
        let mut grid5: [[bool;20];20] = [[true;20];20];
        for row in 7..=20 { grid5[(row-1) as usize][0] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][1] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][2] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][3] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][4] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][5] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][6] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][7] = false; }
        for row in 9..=20 { grid5[(row-1) as usize][8] = false; }
        for row in 11..=13 { grid5[(row-1) as usize][9] = false; }
        for row in 19..=20 { grid5[(row-1) as usize][9] = false; }
        for row in 11..=13 { grid5[(row-1) as usize][10] = false; }
        for row in 15..=17 { grid5[(row-1) as usize][10] = false; }
        for row in 19..=20 { grid5[(row-1) as usize][10] = false; }
        for row in 11..=13 { grid5[(row-1) as usize][11] = false; }
        for row in 15..=17 { grid5[(row-1) as usize][11] = false; }
        for row in 19..=20 { grid5[(row-1) as usize][11] = false; }
        for row in 11..=13 { grid5[(row-1) as usize][12] = false; }
        for row in 15..=17 { grid5[(row-1) as usize][12] = false; }
        for row in 19..=20 { grid5[(row-1) as usize][12] = false; }
        for row in 11..=13 { grid5[(row-1) as usize][13] = false; }
        for row in 19..=20 { grid5[(row-1) as usize][13] = false; }
        for row in 11..=20 { grid5[(row-1) as usize][14] = false; }
        for row in 19..=20 { grid5[(row-1) as usize][15] = false; }
        grid5[20-1][16] = false;
        let pillars = [(10,14),(10,18),(14,14),(14,18)];
        for &(pc, pr) in pillars.iter() { grid5[(pr-1) as usize][(pc-1) as usize] = false; }
        grid5[(16-1) as usize][(10-1) as usize] = false;
        let gray_positions = [(10,15),(11,14),(13,14),(13,18),(11,18),(10,17)];
        for &(gc, gr) in gray_positions.iter() { grid5[(gr-1) as usize][(gc-1) as usize] = false; }
    for row in 15..=17 { grid5[(row-1) as usize][(14-1) as usize] = false; }
        grid5[(14-1) as usize][(12-1) as usize] = false; grid5[(18-1) as usize][(12-1) as usize] = false; // glass
        grid5[(10-1) as usize][(10-1) as usize] = false;
        grid5[2usize][0usize] = false;
        for row in 2..=4 { grid5[(row-1) as usize][1usize] = false; }
        for row in 0..20 {
            for col in 0..20 {
                if !grid5[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                let uc = (col + 1) as i32;
                let ur = (row + 1) as i32;
                let top_mat = if (uc == 6 && (ur == 6 || ur == 7 || ur == 8)) || (uc == 7 && ur == 8) {
                    Some(material_dirt())
                } else {
                    None
                };
                layer5.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: top_mat, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
        }
        for &(pc, pr) in pillars.iter() { layer5.push(Cube { center: glm::vec3((pc-1) as f32 * cube_size, layer5_y, (pr-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pillar(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
        layer5.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (16-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_dark_wood(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        for &(gc, gr) in gray_positions.iter() { layer5.push(Cube { center: glm::vec3((gc-1) as f32 * cube_size, layer5_y, (gr-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
    for row in 15..=18 { layer5.push(Cube { center: glm::vec3((14-1) as f32 * cube_size, layer5_y, (row-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
        layer5.push(Cube { center: glm::vec3((12-1) as f32 * cube_size, layer5_y, (14-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_glass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        layer5.push(Cube { center: glm::vec3((12-1) as f32 * cube_size, layer5_y, (18-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_glass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
    layer5.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (10-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                
        let water_positions = vec![
            (2,2),(2,3),(2,4),
            (3,3),(3,4),(3,5),(3,6),
            (4,2),(4,3),(4,4),(4,5),
            (5,2),(5,3),(5,4),(5,5),(5,6),
            (6,3),(6,4),(6,5),
            (7,3),(7,4),(7,5),(7,6),(7,7),
            (8,2),(8,3),(8,4),(8,5),(8,6),(8,7),
            (9,2),(9,3),(9,4),(9,5),(9,6),(9,7),
            (10,3),(10,4),(10,5),(10,6),(10,7),
            (11,3),(11,4),(11,5),
            (12,5),(12,6),(12,7),(12,8),
            (13,5),(13,6),(13,7),(13,8)
        ];
        for &(wc, wr) in water_positions.iter() {
            let cx = (wc-1) as f32 * cube_size;
            let cz = (wr-1) as f32 * cube_size;
            layer5.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_water(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        }
        for (wc, wr) in water_positions.iter() {
            for i in 1..=3 {
                let y = layer5_y - (i as f32) * 0.85;
                layer5.push(make_small_cube(*wc, *wr, y, material_water()));
            }
            layer5.push(make_small_droplet(*wc, *wr, layer5_y - 0.45, material_water()));
        }
        let reserved = vec![
            (10i32,14i32),(10,18),(14,14),(14,18),(10,16),
            (10,15),(11,14),(13,14),(14,15),(14,16),(14,17),(13,18),(12,18),(11,18),(10,17),(14,18),
            (12,14),(12,18),(10,10),
            (2,2),(2,3),(2,4),(3,3),(3,4),(3,5),(3,6),(4,2),(4,3),(4,4),(4,5),(5,2),(5,3),(5,4),(5,5),(5,6),(6,3),(6,4),(6,5),(7,3),(7,4),(7,5),(7,6),(7,7),(8,2),(8,3),(8,4),(8,5),(8,6),(8,7),(9,2),(9,3),(9,4),(9,5),(9,6),(9,7),(10,3),(10,4),(10,5),(10,6),(10,7),(11,3),(11,4),(11,5),(12,5),(12,6),(12,7),(12,8),(13,5),(13,6),(13,7),(13,8)
        ];
        layer5.retain(|obj| {
            if (obj.center.y - layer5_y).abs() < 1e-3 {
                let gc = (obj.center.x / cube_size).round() as i32 + 1;
                let gr = (obj.center.z / cube_size).round() as i32 + 1;
                let d = obj.material.diffuse;
                if d.r == 80 && d.g == 180 && d.b == 70 && reserved.iter().any(|&(c,r)| c==gc && r==gr) {

        
                    return false;
                }
            }
            true
        });
    let _added = layer5.len() - pre_len;
    {
        let cube_size = 1.0_f32;
        let layer5_y = 4.5 * cube_size;
        let gx = (10 - 1) as f32 * cube_size;
        let gz = (10 - 1) as f32 * cube_size;
        if !layer5.iter().any(|o| (o.center.x - gx).abs() < 1e-3 && (o.center.z - gz).abs() < 1e-3 && (o.center.y - layer5_y).abs() < 1e-3) {
            layer5.push(Cube { center: glm::vec3(gx, layer5_y, gz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin_side(), top_material: Some(material_pumpkin_top()), radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            println!("diag: inserted missing pumpkin at grid (10,10) into layer5");
        }
    }
    }

        vec![
        ("Capa 1 - Suelo".to_string(), layer1),
        ("Capa 2 - Terrazas".to_string(), layer2),
        ("Capa 3 - Casa".to_string(), layer3),
        ("Capa 4 - Detalles".to_string(), layer4),
        ("Capa 5 - Grilla".to_string(), layer5),
        ("Capa 6 - Nivel superior".to_string(), layer6),
        ("Capa 7 - Nivel superior 2".to_string(), layer7),
        ("Capa 8 - Nivel superior 3".to_string(), layer8),
        ("Capa 9 - Nivel superior 4".to_string(), layer9),
        ("Capa 10 - Nivel superior 5".to_string(), layer10),
        ("Capa 11 - Nivel superior 6".to_string(), layer11),
        ("Capa 12 - Nivel superior 7".to_string(), layer12),
        ("Capa 13 - Nivel superior 8".to_string(), layer13),
        ("Capa 14 - Nivel superior 9".to_string(), layer14),
    ]
}


fn reflect(v: &glm::Vec3, n: &glm::Vec3) -> glm::Vec3 {
    *v - *n * 2.0 * glm::dot(v, n)
}

fn sample_sky(dir: &glm::Vec3) -> Color {
    let t = (0.5 * (dir.y + 1.0)).clamp(0.0, 1.0);
    let top = glm::vec3(135.0/255.0, 206.0/255.0, 235.0/255.0);
    let bot = glm::vec3(40.0/255.0, 70.0/255.0, 120.0/255.0);
    let col = top * t + bot * (1.0 - t);
    Color::new((col.x*255.0) as u8, (col.y*255.0) as u8, (col.z*255.0) as u8, 255)
}

fn sample_material(material: &crate::ray_intersect::Material, u: f32, v: f32, face: crate::ray_intersect::FaceId, tx: Option<&texture::TextureManager>) -> Color {
    if let Some(tm) = tx {
        if let Some(ref path) = material.texture {
            if let Some(col) = tm.sample(path.as_str(), u * material.uv_scale, v * material.uv_scale) {
                return col;
            }
        }
        let d = material.diffuse;
        if d.r == 80 && d.g == 180 && d.b == 70 {
            match face {
                crate::ray_intersect::FaceId::Top => {
                    if let Some(col) = tm.sample("cesped.png", u * material.uv_scale, v * material.uv_scale) {
                        return col;
                    }
                }
                crate::ray_intersect::FaceId::Left | crate::ray_intersect::FaceId::Right | crate::ray_intersect::FaceId::Front | crate::ray_intersect::FaceId::Back => {
                    if let Some(col) = tm.sample("cesped_de_lado.png", u * material.uv_scale, v * material.uv_scale) {
                        return col;
                    }
                }
                _ => {}
            }
        }
    }

    material.diffuse
}

fn make_small_cube(col: i32, row: i32, y: f32, mat: crate::ray_intersect::Material) -> Cube {
    let cube_size = 1.0_f32;
    let cx = (col - 1) as f32 * cube_size;
    let cz = (row - 1) as f32 * cube_size;
    Cube { center: glm::vec3(cx, y, cz), half_size: glm::vec3(0.3, 0.3, 0.3), rot_y: 0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.3,0.3,0.3)) }
}

fn make_small_droplet(col: i32, row: i32, y: f32, mat: crate::ray_intersect::Material) -> Cube {
    let cube_size = 1.0_f32;
    let cx = (col - 1) as f32 * cube_size + 0.12; 
    let cz = (row - 1) as f32 * cube_size - 0.08;
    Cube { center: glm::vec3(cx, y, cz), half_size: glm::vec3(0.18, 0.18, 0.18), rot_y: 0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.18,0.18,0.18)) }
}



fn cast_ray(cam_orig: &glm::Vec3, dir: &glm::Vec3, objects: &[Cube], bvh: Option<&BVH>, tx: Option<&texture::TextureManager>) -> Color {
    cast_ray_rec(cam_orig, dir, objects, bvh, 0, tx)
}

fn cast_ray_rec(cam_orig: &glm::Vec3, dir: &glm::Vec3, objects: &[Cube], bvh: Option<&BVH>, depth: i32, tx: Option<&texture::TextureManager>) -> Color {
    
    let mut intersect = crate::ray_intersect::Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    
    if let Some(b) = bvh {
        intersect = intersect_bvh(b, objects, cam_orig, dir);
        if !intersect.is_intersecting {
            return sample_sky(dir);
        }
    } else {
        for object in objects {
            let tmp = object.ray_intersect(cam_orig, dir);
            if tmp.is_intersecting && tmp.distance < zbuffer {
                zbuffer = tmp.distance;
                intersect = tmp;
            }
        }
        if !intersect.is_intersecting {
            return sample_sky(dir);
        }
    }

    
    if depth == 0 {
        HIT_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    let isect = intersect;
    let light_dir = glm::normalize(&glm::vec3(-0.6, 0.9, -0.4));
    let diff = glm::dot(&isect.normal, &light_dir).max(0.0);
    let ambient = 0.36; 
    let intensity = (ambient + diff * 1.0).min(1.0);

    
    let (u, v) = isect.uv;
    let base_col = sample_material(&isect.material, u, v, isect.face, tx);
    
    fn srgb_to_linear(c: Color) -> glm::Vec3 {
        let sr = c.r as f32 / 255.0;
        let sg = c.g as f32 / 255.0;
        let sb = c.b as f32 / 255.0;
        glm::vec3(sr.powf(2.2), sg.powf(2.2), sb.powf(2.2))
    }
    let base_f = srgb_to_linear(base_col);

    
    let _reflectivity = isect.material.reflectivity;
    let _transparency = isect.material.transparency;
    let mut final_color = base_f * intensity;

    
    let bias = 1e-3f32;
    let _offset_point = isect.point + isect.normal * bias;

    
    let view_dir = glm::normalize(&(-*dir));
    let _cos_theta = glm::dot(&view_dir, &isect.normal).max(0.0);

    
    let mut is_water = false;
    let mat_col_check = isect.material.diffuse;
    if mat_col_check.r == 64 && mat_col_check.g == 160 && mat_col_check.b == 255 {
        is_water = true;
    }
    if let Some(ref path) = isect.material.texture {
        if path.contains("water") { is_water = true; }
    }

    
    let spec_strength = isect.material.specular;
    if spec_strength > 0.0 {
        let reflect_light = glm::normalize(&reflect(&-light_dir, &isect.normal));
        let spec_angle = glm::dot(&view_dir, &reflect_light).max(0.0);
        let spec = spec_strength * spec_angle.powf(32.0);
        final_color += glm::vec3(spec, spec, spec);
    }

    let gamma = 1.0 / 2.2;
    let mat_col = isect.material.diffuse;
    let is_darkwood_mat = mat_col.r == 0x7F && mat_col.g == 0x66 && mat_col.b == 0x45;
    if is_darkwood_mat {
        return base_col;
    }

    if isect.material.transparency > 0.01 {
    let sky_col = sample_sky(dir);
        let sky_f = glm::vec3(sky_col.r as f32 / 255.0, sky_col.g as f32 / 255.0, sky_col.b as f32 / 255.0);
    let trans = isect.material.transparency.clamp(0.0, 1.0);
        let cont_origin = isect.point + (*dir) * (bias * 10.0);
        let cont_dir = *dir;
        let mut cont_col_f: Option<glm::Vec3> = None;
        let mut best = crate::ray_intersect::Intersect::empty();
        let mut best_dist = f32::INFINITY;
        for obj in objects.iter() {
            if obj.half_size.x < 0.45 { continue; }
            let tmp = obj.ray_intersect(&cont_origin, &cont_dir);
            if tmp.is_intersecting && tmp.distance < best_dist {
                best_dist = tmp.distance; best = tmp;
            }
        }
        if best.is_intersecting {
            let cont_base = sample_material(&best.material, best.uv.0, best.uv.1, best.face, tx);
            cont_col_f = Some(srgb_to_linear(cont_base));
        }


        let env = if let Some(cf) = cont_col_f { cf } else { sky_f };

        let mut blended = base_f * (1.0 - trans) + env * trans;

        let refl = isect.material.reflectivity.clamp(0.0, 1.0);
        if refl > 0.01 && depth < 3 {
            let bias = 1e-3f32;
            let reflect_dir = glm::normalize(&reflect(dir, &isect.normal));
            let reflect_origin = isect.point + isect.normal * bias;
            let refl_col_srgb = cast_ray_rec(&reflect_origin, &reflect_dir, objects, bvh, depth + 1, tx);
            let refl_col_f = srgb_to_linear(refl_col_srgb);
            blended = blended * (1.0 - refl) + refl_col_f * refl;
        }

    
        if is_water {
            blended = blended * 0.9 + glm::vec3(0.0, 0.03, 0.08) * 0.1;
        }

        let mut mapped = blended * intensity * EXPOSURE;
        mapped = glm::vec3(
            mapped.x / (1.0 + mapped.x),
            mapped.y / (1.0 + mapped.y),
            mapped.z / (1.0 + mapped.z),
        );
        let gamma = 1.0 / 2.2;
        let r = (mapped.x.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
        let g = (mapped.y.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
        let b = (mapped.z.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
        return Color::new(r, g, b, 255);
    }
    let refl = isect.material.reflectivity.clamp(0.0, 1.0);
    if refl > 0.01 && depth < 3 {
        let bias = 1e-3f32;
        let reflect_dir = glm::normalize(&reflect(dir, &isect.normal));
        let reflect_origin = isect.point + isect.normal * bias;
        let refl_col_srgb = cast_ray_rec(&reflect_origin, &reflect_dir, objects, bvh, depth + 1, tx);
        let refl_col_f = srgb_to_linear(refl_col_srgb);
        final_color = final_color * (1.0 - refl) + refl_col_f * refl;
    }
    final_color *= EXPOSURE;
    let mut mapped = final_color;
    mapped = glm::vec3(
        mapped.x / (1.0 + mapped.x),
        mapped.y / (1.0 + mapped.y),
        mapped.z / (1.0 + mapped.z),
    );
    let gamma = 1.0 / 2.2;
    let r = (mapped.x.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
    let g = (mapped.y.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
    let b = (mapped.z.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
    Color::new(r, g, b, 255)
}

fn render(framebuffer: &mut Framebuffer, objects: &[Cube], cam_pos: &glm::Vec3, cam_yaw: f32, cam_pitch: f32, bvh: Option<&BVH>, tx: Option<&texture::TextureManager>) {
    let mut render_scale: usize = 2;
    if framebuffer.width() > 1200 || framebuffer.height() > 1200 { render_scale = 3; }
    let w = (framebuffer.width() as usize / render_scale).max(1);
    let h = (framebuffer.height() as usize / render_scale).max(1);

    let width_f = w as f32;
    let height_f = h as f32;
    let aspect = width_f / height_f;
    let fov: f32 = 60f32.to_radians();
    let scale = (fov * 0.5).tan();

    let num_pixels = w * h;
    let colors: Vec<Color> = (0..num_pixels).into_par_iter().map(|idx| {
        let i = idx % w;
        let j = idx / w;
        let px = (2.0 * (i as f32 + 0.5) / width_f - 1.0) * aspect * scale;
        let py = (1.0 - 2.0 * (j as f32 + 0.5) / height_f) * scale;
    let cp = cam_pitch.cos();
    let sp = cam_pitch.sin();
    let cy = cam_yaw.cos();
    let sy = cam_yaw.sin();
    let cam_forward = glm::vec3(-sy * cp, sp, -cy * cp);
    let world_up = glm::vec3(0.0, 1.0, 0.0);
    let mut cam_right = glm::cross(&cam_forward, &world_up);
    if glm::length(&cam_right) < 1e-6 { cam_right = glm::vec3(1.0, 0.0, 0.0); }
    cam_right = glm::normalize(&cam_right);
    let mut cam_up = glm::cross(&cam_right, &cam_forward);
    if glm::length(&cam_up) < 1e-6 { cam_up = glm::vec3(0.0, 1.0, 0.0); }
    let cam_up = glm::normalize(&cam_up);

    let ray_camera = glm::vec3(px, py, -1.0);
    let mut ray_world = cam_right * ray_camera.x + cam_up * ray_camera.y + cam_forward * (-ray_camera.z);
    ray_world = glm::normalize(&ray_world);
        cast_ray(cam_pos, &ray_world, objects, bvh, tx)
    }).collect();

    for j in 0..h {
        for i in 0..w {
            let col = colors[j * w + i];
            let dst_x = (i * render_scale) as u32;
            let dst_y = (j * render_scale) as u32;
            framebuffer.set_current_color(col);
            for oy in 0..render_scale {
                for ox in 0..render_scale {
                    let px = dst_x + ox as u32;
                    let py = dst_y + oy as u32;
                    if px < framebuffer.width() && py < framebuffer.height() {
                        framebuffer.set_pixel(px, py);
                    }
                }
            }
        }
    }
}

fn main() {
    const WIN_W: i32 = 900;
    const WIN_H: i32 = 900;

    let (mut rl, thread) = raylib::init().size(WIN_W, WIN_H).title("Escena de Minecraft - Ihan Marroquin").build();

    let mut fb = Framebuffer::new(WIN_W as u32, WIN_H as u32, Color::BLACK);
    // cámara: posición y yaw (rotación Y)
    // Inicio: posición de cámara desde una esquina
    let mut cam_pos = glm::vec3(-12.5, 5.5, -10.5);
    let mut cam_yaw: f32;
    let mut cam_pitch: f32;

    let args: Vec<String> = std::env::args().collect();

    // Construir escena desde el constructor determinista. Opcional: --layer N
    let scene_objects: Vec<Cube> = {
        let mut selected_layer: Option<usize> = None;
        for a in args.iter() {
            if a.starts_with("--layer=") {
                if let Some(eq) = a.split_once('=').map(|x| x.1) { if let Ok(n) = eq.parse::<usize>() { selected_layer = Some(n); break; } }
            }
        }
        if selected_layer.is_none() {
            if let Some(pos) = args.iter().position(|a| a == "--layer") {
                if pos + 1 < args.len() { if let Ok(n) = args[pos+1].parse::<usize>() { selected_layer = Some(n); } }
            }
        }

        if let Some(n) = selected_layer {
            let layers = build_reference_diorama_layers();
            if n >= 1 && n <= layers.len() {                
                layers.into_iter().nth(n-1).unwrap().1
            } else {
                build_reference_diorama()
            }
        } else {
            build_reference_diorama()
        }
    };
    // Depuración: listar objetos cerca de las escaleras
    for o in scene_objects.iter() {
        let cx = o.center.x;
        let cz = o.center.z;
        if (8.0..10.0).contains(&cx) && (14.0..17.0).contains(&cz) {
            let d = o.material.diffuse;
        }
    }
    // Depuración adicional: inspeccionar objetos cerca de la pared gris
    for o in scene_objects.iter() {
        let cx = o.center.x;
        let cz = o.center.z;
        if (13.0..15.0).contains(&cx) && (14.0..=18.0).contains(&cz) {
            let d = o.material.diffuse;
        }
    }
    // Conteo diagnóstico de materiales para verificar colocaciones
    {
        let mut grass_c = 0usize;
        let mut stone_c = 0usize;
        let mut pillar_c = 0usize;
        let mut dark_c = 0usize;
        let mut path_c = 0usize;
        for o in scene_objects.iter() {
            let c = o.material.diffuse;
            match (c.r, c.g, c.b) {
                (80,180,70) => grass_c += 1,
                (0x66,0x68,0x66) => stone_c += 1,
                (0xAF,0x9D,0x7B) => pillar_c += 1,
                (0x7F,0x66,0x45) => dark_c += 1,
                (218,187,147) => path_c += 1,
                _ => {}
            }
        }
    }
    let bvh = if !scene_objects.is_empty() { Some(build_bvh(&scene_objects)) } else { None };

    // Crear y cargar texturas (módulo de texturas simple)
    let mut texture_manager = texture::TextureManager::new("texturas");
    if let Err(e) = texture_manager.load("cesped.png") {
        eprintln!("warning: failed to load cesped.png: {}", e);
    }
    if let Err(e) = texture_manager.load("cesped_de_lado.png") {
        eprintln!("warning: failed to load cesped_de_lado.png: {}", e);
    }
    if let Err(e) = texture_manager.load("pared_gris.png") {
        eprintln!("warning: failed to load pared_gris.png: {}", e);
    }
    if let Err(e) = texture_manager.load("pilar.png") {
        eprintln!("warning: failed to load pilar.png: {}", e);
    }
    if let Err(e) = texture_manager.load("oakwood.png") {
        eprintln!("warning: failed to load oakwood.png: {}", e);
    }
    if let Err(e) = texture_manager.load("camino_de_lado.png") {
        eprintln!("warning: failed to load camino_de_lado.png: {}", e);
    }
    if let Err(e) = texture_manager.load("arriba_calabaza.png") {
        eprintln!("warning: failed to load arriba_calabaza.png: {}", e);
    }

    if let Err(e) = texture_manager.load("calabaza.png") {
        eprintln!("warning: failed to load calabaza.png: {}", e);
    }

    for o in scene_objects.iter() {
        if let Some(ref path) = o.material.texture {
            if path == "calabaza.png" {
                println!("diag: pumpkin side at x={} z={} y={} material.texture={}", o.center.x, o.center.z, o.center.y, path);
            }
        }
        if let Some(ref tm) = o.top_material {
            if let Some(ref tpath) = tm.texture {
                if tpath == "arriba_calabaza.png" {
                    println!("diag: pumpkin top at x={} z={} y={} top_material.texture={}", o.center.x, o.center.z, o.center.y, tpath);
                }
            }
        }
    }

    // Renderizar un frame de inicio con la rotación optimizada
    cam_yaw = -2.490465_f32;
    if let Err(e) = texture_manager.load("camino.png") {
        eprintln!("warning: failed to load camino.png: {}", e);
    }
    cam_pitch = -0.549000_f32;

    // Opcional: simular una pulsación de Q al inicio para ajustar pitch
    const APPLY_STARTUP_Q: bool = true;
    const STARTUP_Q_SECONDS: f32 = 0.35; 
    const SIM_ROT_SPEED: f32 = 1.6_f32;
    if APPLY_STARTUP_Q {
        let delta = SIM_ROT_SPEED * STARTUP_Q_SECONDS;
        cam_pitch += delta;
    }

    // parámetros de movimiento
    let move_speed = 2.6_f32; // unidades por segundo (ajusta)
    let rot_speed = 1.6_f32; // rad/s para girar escena con A/D

    // Precompute render-ready scene and BVH to avoid per-frame cloning
    let mut render_scene_objects = scene_objects.clone();
    for obj in render_scene_objects.iter_mut() {
        if let Some(tm) = &obj.top_material {
            if let Some(ref path) = tm.texture {
                if path == "camino.png" {
                    obj.material = material_dirt_path_side();
                }
            }
        }
    }
    let render_bvh = if !render_scene_objects.is_empty() { Some(build_bvh(&render_scene_objects)) } else { None };
    let mut auto_rotate = false;
    const DIAG_FRAME_WINDOW: usize = 10;
    let mut diag_frame_counter: usize = 0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

    // Entrada: movimiento relativo a orientación de cámara
        let yaw = cam_yaw;
        let pitch = cam_pitch;
        let cp = pitch.cos();
        let sp = pitch.sin();
        let cy = yaw.cos();
        let sy = yaw.sin();
        let forward = glm::vec3(-sy * cp, sp, -cy * cp);
        let world_up = glm::vec3(0.0, 1.0, 0.0);
        let mut right = glm::cross(&forward, &world_up);
        if glm::length(&right) < 1e-6 { right = glm::vec3(1.0, 0.0, 0.0); }
        right = glm::normalize(&right);

        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            cam_pos -= right * move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            cam_pos += right * move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            cam_pos += forward * move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            cam_pos -= forward * move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_W) {
            cam_pos.y += move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            cam_pos.y -= move_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            cam_yaw -= rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            cam_yaw += rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            cam_pitch -= rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            cam_pitch += rot_speed * dt;
        }

        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01_f32;
        if cam_pitch > max_pitch { cam_pitch = max_pitch; }
        if cam_pitch < -max_pitch { cam_pitch = -max_pitch; }

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            auto_rotate = !auto_rotate;
        }
        if auto_rotate {
            cam_yaw += 0.2 * dt;
        }

        if rl.is_key_down(KeyboardKey::KEY_Z) || rl.is_key_down(KeyboardKey::KEY_PAGE_UP) {
            cam_pos.z += -move_speed * dt * 0.5;
        }
        if rl.is_key_down(KeyboardKey::KEY_X) || rl.is_key_down(KeyboardKey::KEY_PAGE_DOWN) {
            cam_pos.z += move_speed * dt * 0.5;
        }

        fb.clear(Color::BLACK);
    render(&mut fb, &render_scene_objects, &cam_pos, cam_yaw, cam_pitch, render_bvh.as_ref(), Some(&texture_manager));
        diag_frame_counter += 1;
        if diag_frame_counter >= DIAG_FRAME_WINDOW {
            diag_frame_counter = 0;
            let hits = HIT_COUNT.swap(0, Ordering::Relaxed);
            println!("[diagnostic] hits_last_{DIAG_FRAME_WINDOW}_frames = {hits}");
        }
    
        fb.present(&mut rl, &thread, 1.0);
    }
}

