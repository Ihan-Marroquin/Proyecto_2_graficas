mod framebuffer;
mod ray_intersect;
mod cube;
mod materials;
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

// Construye el diorama de referencia.
#[allow(clippy::needless_range_loop)]
fn build_reference_diorama() -> Vec<Cube> {
    // use materials::*; // redundant: top-level import already present
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
            let top_mat = if is_brown_top { Some(material_path()) } else { None };
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
            let top_mat = if is_brown_top { Some(material_path()) } else { None };
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
                    top_mat = Some(material_path());
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
                top_mat = Some(material_path());
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
        // (no depender de captures/reference.png)
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
                    v.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y: 0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
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
            v.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (10-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
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

    for obj in all.into_iter() {
        let y = obj.center.y;
        if (y - 0.5).abs() < 0.4 {
            layer1.push(obj);
        } else if (y - 1.5).abs() < 0.4 {
            layer2.push(obj);
        } else if y <= 2.6 {
            layer3.push(obj);
        } else if y <= 3.6 {
            layer4.push(obj);
        } else {
            layer5.push(obj);
        }
    }

    // Siempre construir layer5 de forma determinista, sin depender de ficheros externos
    {
    // use materials::*; // import not required here (module-level import present)
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
        grid5[(16-1) as usize][(10-1) as usize] = false; // darkwood
        let gray_positions = [(10,15),(11,14),(13,14),(13,18),(11,18),(10,17)];
        for &(gc, gr) in gray_positions.iter() { grid5[(gr-1) as usize][(gc-1) as usize] = false; }
    for row in 15..=17 { grid5[(row-1) as usize][(14-1) as usize] = false; }
        grid5[(14-1) as usize][(12-1) as usize] = false; grid5[(18-1) as usize][(12-1) as usize] = false; // glass
        grid5[(10-1) as usize][(10-1) as usize] = false; // pumpkin
        // water cells: (3,1) and rows 2..=4 at column 2
        grid5[2usize][0usize] = false;
        for row in 2..=4 { grid5[(row-1) as usize][1usize] = false; } // water
        for row in 0..20 {
            for col in 0..20 {
                if !grid5[row][col] { continue; }
                let cx = col as f32 * cube_size;
                let cz = row as f32 * cube_size;
                layer5.push(Cube { center: glm::vec3(cx, layer5_y, cz), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_grass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
            }
        }
        for &(pc, pr) in pillars.iter() { layer5.push(Cube { center: glm::vec3((pc-1) as f32 * cube_size, layer5_y, (pr-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pillar(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
        layer5.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (16-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_dark_wood(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        for &(gc, gr) in gray_positions.iter() { layer5.push(Cube { center: glm::vec3((gc-1) as f32 * cube_size, layer5_y, (gr-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
    for row in 15..=18 { layer5.push(Cube { center: glm::vec3((14-1) as f32 * cube_size, layer5_y, (row-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_light_gray(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) }); }
        layer5.push(Cube { center: glm::vec3((12-1) as f32 * cube_size, layer5_y, (14-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_glass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        layer5.push(Cube { center: glm::vec3((12-1) as f32 * cube_size, layer5_y, (18-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_glass(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
        layer5.push(Cube { center: glm::vec3((10-1) as f32 * cube_size, layer5_y, (10-1) as f32 * cube_size), half_size: glm::vec3(0.5,0.5,0.5), rot_y:0.0, material: material_pumpkin(), top_material: None, radius: glm::length(&glm::vec3(0.5,0.5,0.5)) });
                
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
    }

    vec![
        ("Capa 1 - Suelo".to_string(), layer1),
        ("Capa 2 - Terrazas".to_string(), layer2),
        ("Capa 3 - Casa".to_string(), layer3),
        ("Capa 4 - Detalles".to_string(), layer4),
        ("Capa 5 - Grilla".to_string(), layer5),
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

fn sample_material(material: &crate::ray_intersect::Material, _u: f32, _v: f32) -> Color {
    material.diffuse
}

// Small helper constructors used to simulate falling water droplets/columns.
fn make_small_cube(col: i32, row: i32, y: f32, mat: crate::ray_intersect::Material) -> Cube {
    let cube_size = 1.0_f32;
    let cx = (col - 1) as f32 * cube_size;
    let cz = (row - 1) as f32 * cube_size;
    Cube { center: glm::vec3(cx, y, cz), half_size: glm::vec3(0.3, 0.3, 0.3), rot_y: 0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.3,0.3,0.3)) }
}

fn make_small_droplet(col: i32, row: i32, y: f32, mat: crate::ray_intersect::Material) -> Cube {
    let cube_size = 1.0_f32;
    let cx = (col - 1) as f32 * cube_size + 0.12; // slight offset for variation
    let cz = (row - 1) as f32 * cube_size - 0.08;
    Cube { center: glm::vec3(cx, y, cz), half_size: glm::vec3(0.18, 0.18, 0.18), rot_y: 0.0, material: mat, top_material: None, radius: glm::length(&glm::vec3(0.18,0.18,0.18)) }
}



fn cast_ray(cam_orig: &glm::Vec3, dir: &glm::Vec3, objects: &[Cube], bvh: Option<&BVH>) -> Color {
    cast_ray_rec(cam_orig, dir, objects, bvh, 0)
}

fn cast_ray_rec(cam_orig: &glm::Vec3, dir: &glm::Vec3, objects: &[Cube], bvh: Option<&BVH>, depth: i32) -> Color {
    
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
    let ambient = 0.48; 
    let intensity = (ambient + diff * 0.92).min(1.0);

    
    let (u, v) = isect.uv;
    let base_col = sample_material(&isect.material, u, v);

    

    
    let base_f = glm::vec3(base_col.r as f32 / 255.0, base_col.g as f32 / 255.0, base_col.b as f32 / 255.0);

    
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

    // no debug capture modes: always continue shading

   
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
            let cont_base = sample_material(&best.material, best.uv.0, best.uv.1);
            cont_col_f = Some(glm::vec3(cont_base.r as f32 / 255.0, cont_base.g as f32 / 255.0, cont_base.b as f32 / 255.0));
        }

        let env = if let Some(cf) = cont_col_f { cf } else { sky_f };

      
        let mut blended = base_f * (1.0 - trans) + env * trans;

    
        if is_water {
            blended = blended * 0.9 + glm::vec3(0.0, 0.03, 0.08) * 0.1;
        }

        let out = blended * intensity;
        let gamma = 1.0 / 2.2;
        let r = (out.x.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
        let g = (out.y.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
        let b = (out.z.clamp(0.0, 1.0).powf(gamma) * 255.0) as u8;
        return Color::new(r, g, b, 255);
    }

    let bands: f32 = 4.0;
    let quant = (final_color * bands).map(|v| v.floor()) / bands;
    let view_dot = glm::dot(&view_dir, &isect.normal).abs();
    let mut edge_strength = ((1.0 - view_dot) - 0.02).max(0.0) / (0.6 - 0.02); 
    edge_strength = edge_strength.min(1.0);
    let stylized = quant * (1.0 - 0.06 * edge_strength);
    let corrected = glm::vec3(
        stylized.x.clamp(0.0, 1.0).powf(gamma),
        stylized.y.clamp(0.0, 1.0).powf(gamma),
        stylized.z.clamp(0.0, 1.0).powf(gamma),
    );
    let outline_strength = (edge_strength * 0.9).min(1.0);
    let outlined = corrected * (1.0 - outline_strength * 0.5);
    let r = (outlined.x * 255.0) as u8;
    let g = (outlined.y * 255.0) as u8;
    let b = (outlined.z * 255.0) as u8;
    Color::new(r, g, b, 255)
}

fn render(framebuffer: &mut Framebuffer, objects: &[Cube], cam_pos: &glm::Vec3, cam_yaw: f32, cam_pitch: f32, bvh: Option<&BVH>) {
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
        cast_ray(cam_pos, &ray_world, objects, bvh)
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

    // Sin presets automáticos: renderizar desde la cámara configurada (sin movimientos)
    println!("[camera] using initial camera without translations or nudges");

    // Imprimir coordenadas iniciales (X, Y, Z)
    println!("Initial camera position: x={:.3}, y={:.3}, z={:.3}", cam_pos.x, cam_pos.y, cam_pos.z);

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
                println!("[diag] rendering only layer {} -> {}", n, layers[n-1].0);
                
                layers.into_iter().nth(n-1).unwrap().1
            } else {
                println!("[diag] requested layer {} out of range (1..={}), using full scene", n, layers.len());
                build_reference_diorama()
            }
        } else {
            build_reference_diorama()
        }
    };
    // Depuración: listar objetos cerca de las escaleras
    println!("[debug] scanning for objects near stairs (x in [8,10), z in [14,17))");
    for o in scene_objects.iter() {
        let cx = o.center.x;
        let cz = o.center.z;
        if (8.0..10.0).contains(&cx) && (14.0..17.0).contains(&cz) {
            let d = o.material.diffuse;
            println!("[debug] obj center=({:.2},{:.2},{:.2}) half=({:.2},{:.2},{:.2}) diffuse=({},{},{})", cx, o.center.y, cz, o.half_size.x, o.half_size.y, o.half_size.z, d.r, d.g, d.b);
        }
    }
    // Depuración adicional: inspeccionar objetos cerca de la pared gris
    println!("[debug] scanning for objects near gray wall (approx col 14, rows 15..18)");
    for o in scene_objects.iter() {
        let cx = o.center.x;
        let cz = o.center.z;
        // world col 14 -> x in [13.0, 15.0), rows 15..18 -> z in [14.0, 18.0]
        if (13.0..15.0).contains(&cx) && (14.0..=18.0).contains(&cz) {
            let d = o.material.diffuse;
            println!("[debug-wall] obj center=({:.2},{:.2},{:.2}) y={} mat=({},{},{}) half=({:.2},{:.2},{:.2})", cx, o.center.y, cz, o.center.y, d.r, d.g, d.b, o.half_size.x, o.half_size.y, o.half_size.z);
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
        println!("[diag] counts: grass={grass_c} stone={stone_c} pillar={pillar_c} darkwood={dark_c} path={path_c}");
    }
    let bvh = if !scene_objects.is_empty() { Some(build_bvh(&scene_objects)) } else { None };

    // Renderizar un frame de inicio con la rotación optimizada
    cam_yaw = -2.490465_f32;
    cam_pitch = -0.549000_f32;

    // Opcional: simular una pulsación de Q al inicio para ajustar pitch
    const APPLY_STARTUP_Q: bool = true;
    const STARTUP_Q_SECONDS: f32 = 0.35; 
    const SIM_ROT_SPEED: f32 = 1.6_f32;
    if APPLY_STARTUP_Q {
        let delta = SIM_ROT_SPEED * STARTUP_Q_SECONDS;
        cam_pitch += delta;
        println!("[startup] applied simulated Q rotation +{delta:.6} rad -> cam_pitch {cam_pitch:.6}");
    }

    // Sin traslaciones al inicio
    println!("[camera] using optimized startup rotation -> yaw {cam_yaw:.6} pitch {cam_pitch:.6}");
    println!("Adjusted camera position: x={:.3}, y={:.3}, z={:.3}", cam_pos.x, cam_pos.y, cam_pos.z);

    // parámetros de movimiento
    let move_speed = 2.6_f32; // unidades por segundo (ajusta)
    let rot_speed = 1.6_f32; // rad/s para girar escena con A/D

    let mut auto_rotate = false;
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
        // A/D giran la escena (ajustan yaw)
        if rl.is_key_down(KeyboardKey::KEY_A) {
            cam_yaw -= rot_speed * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            cam_yaw += rot_speed * dt;
        }
        // Q/E ajustan pitch (rotación abajo/arriba) - now Q rotates down, E rotates up
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            cam_pitch -= rot_speed * dt; // rotate down
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            cam_pitch += rot_speed * dt; // rotate up
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

        // Zoom controls: Z/X or PageUp/PageDown
        if rl.is_key_down(KeyboardKey::KEY_Z) || rl.is_key_down(KeyboardKey::KEY_PAGE_UP) {
            cam_pos.z += -move_speed * dt * 0.5; // zoom in
        }
        if rl.is_key_down(KeyboardKey::KEY_X) || rl.is_key_down(KeyboardKey::KEY_PAGE_DOWN) {
            cam_pos.z += move_speed * dt * 0.5; // zoom out
        }

        fb.clear(Color::BLACK);
    let objects = &scene_objects;
    render(&mut fb, objects, &cam_pos, cam_yaw, cam_pitch, bvh.as_ref());
        let hits = HIT_COUNT.swap(0, Ordering::Relaxed);
        println!("[diagnostic] hits_this_frame = {hits}");
    

    

        fb.present(&mut rl, &thread, 1.0);
    }
}

