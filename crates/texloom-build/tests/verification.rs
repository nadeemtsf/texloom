use texloom_build::verification::*;

fn bounds(x: f64, y: f64) -> Rect {
    Rect {
        x,
        y,
        width: 2.0,
        height: 3.0,
    }
}
fn paint(view: &mut Inspection) {
    for r in &mut view.rasters {
        r.pixels.fill(255);
    }
    for g in &view.glyphs {
        let r = &mut view.rasters[g.page];
        for y in
            (g.bounds.y * r.scale) as usize..((g.bounds.y + g.bounds.height) * r.scale) as usize
        {
            for x in
                (g.bounds.x * r.scale) as usize..((g.bounds.x + g.bounds.width) * r.scale) as usize
            {
                let offset = y * r.stride + x * r.channels;
                r.pixels[offset..offset + 3].fill(0);
            }
        }
    }
}
fn baseline() -> Inspection {
    let mut result = Inspection {
        pages: vec![Page {
            width: 80.0,
            height: 60.0,
            crop: [0.0, 0.0, 80.0, 60.0],
            rotation: 0,
            annotations: 0,
        }],
        glyphs: vec![
            Glyph {
                text: "A".into(),
                page: 0,
                bounds: bounds(10.0, 10.0),
                target: true,
            },
            Glyph {
                text: "A".into(),
                page: 0,
                bounds: bounds(50.0, 10.0),
                target: false,
            },
        ],
        graphics: vec![],
        rasters: vec![Raster {
            width: 160,
            height: 120,
            stride: 640,
            channels: 4,
            scale: 2.0,
            pixels: vec![255; 640 * 120],
        }],
    };
    paint(&mut result);
    result
}
fn moved(a: &Inspection, dx: f64, dy: f64) -> Inspection {
    let mut b = a.clone();
    for g in &mut b.glyphs {
        if g.target {
            g.bounds.x += dx;
            g.bounds.y += dy;
        }
    }
    paint(&mut b);
    b
}
fn compare_move(a: &Inspection, b: &Inspection) -> Result<Verified> {
    compare(a, b, Operation::Displace { dx: 3.0, dy: 8.0 })
}

#[test]
fn exact_translation_preserves_repeated_unselected_text_and_all_pixels() {
    let a = baseline();
    let mut b = moved(&a, 3.0, 8.0);
    b.glyphs.reverse(); // Do not depend on extraction order.
    let proof = compare_move(&a, &b).unwrap();
    assert_eq!(proof.target_glyphs, 1);
    assert_eq!(proof.glyphs, 2);
    assert_eq!(proof.pixels, 160 * 120);
}
#[test]
fn identity_checks_every_pixel_even_inside_selected_glyphs() {
    let a = baseline();
    assert!(compare(&a, &a, Operation::Identity).is_ok());
    let mut b = a.clone();
    b.rasters[0].pixels[20 * 640 + 20 * 4] = 255;
    assert!(compare(&a, &b, Operation::Identity).is_err());
}
#[test]
fn a_mask_cannot_hide_duplicate_ink_in_the_vacated_position() {
    let a = baseline();
    let mut b = moved(&a, 3.0, 8.0);
    for y in 20..26 {
        for x in 20..24 {
            let n = y * 640 + x * 4;
            b.rasters[0].pixels[n..n + 3].fill(0);
        }
    }
    assert_eq!(
        compare_move(&a, &b).unwrap_err(),
        "Raster appearance changed beyond the exact operation"
    );
}
#[test]
fn target_appearance_and_unrelated_pixels_are_both_checked() {
    let a = baseline();
    for n in [36 * 640 + 26 * 4, 80 * 640 + 100 * 4] {
        let mut b = moved(&a, 3.0, 8.0);
        b.rasters[0].pixels[n] = 127;
        assert!(compare_move(&a, &b).is_err());
    }
}
#[test]
fn moved_neighbors_changed_text_or_target_identity_fail() {
    let a = baseline();
    for change in 0..4 {
        let mut b = moved(&a, 3.0, 8.0);
        match change {
            0 => b.glyphs[1].bounds.y += 0.021,
            1 => b.glyphs[0].text = "B".into(),
            2 => b.glyphs[0].target = false,
            _ => {
                b.glyphs.pop();
            }
        }
        assert!(compare_move(&a, &b).is_err());
    }
}
#[test]
fn coincident_ligature_records_preserve_multiplicity() {
    let mut a = baseline();
    a.glyphs.push(a.glyphs[0].clone());
    let mut b = moved(&a, 3.0, 8.0);
    assert!(compare_move(&a, &b).is_ok());
    b.glyphs.pop();
    assert!(compare_move(&a, &b).is_err());
}
#[test]
fn source_and_destination_windows_cannot_cover_other_text_or_graphics() {
    for at_destination in [false, true] {
        for graphic in [false, true] {
            let mut a = baseline();
            let rect = bounds(
                if at_destination { 13.0 } else { 10.0 },
                if at_destination { 18.0 } else { 10.0 },
            );
            if graphic {
                a.graphics.push(Graphic {
                    kind: 2,
                    page: 0,
                    bounds: rect,
                });
            } else {
                a.glyphs[1].bounds = rect;
            }
            paint(&mut a);
            let b = moved(&a, 3.0, 8.0);
            assert!(compare_move(&a, &b).is_err());
        }
    }
}
#[test]
fn annotations_and_nonstandard_page_coordinates_fail_closed() {
    let a = baseline();
    for change in 0..5 {
        let mut b = a.clone();
        match change {
            0 => b.pages[0].annotations = 1,
            1 => b.pages[0].rotation = 90,
            2 => b.pages[0].crop[0] = 1.0,
            3 => b.pages[0].width = f64::NAN,
            _ => b.pages[0].crop[2] -= 1.0,
        }
        assert!(compare(&a, &b, Operation::Identity).is_err());
    }
}
#[test]
fn all_pages_are_compared_and_cross_page_targets_are_refused() {
    let mut a = baseline();
    a.pages.push(a.pages[0].clone());
    a.rasters.push(a.rasters[0].clone());
    let mut b = a.clone();
    b.rasters[1].pixels[0] = 0;
    assert!(compare(&a, &b, Operation::Identity).is_err());
    let mut other = a.glyphs[0].clone();
    other.page = 1;
    a.glyphs.push(other);
    paint(&mut a);
    assert!(compare_move(&a, &moved(&a, 3.0, 8.0)).is_err());
}
#[test]
fn unsupported_fractional_shifts_and_off_page_destinations_are_refused() {
    let a = baseline();
    for (dx, dy) in [(0.1, 8.0), (-20.0, 0.0), (0.0, 100.0), (f64::NAN, 0.0)] {
        let b = moved(
            &a,
            if dx.is_finite() { dx } else { 0.0 },
            if dy > 50.0 { 0.0 } else { dy },
        );
        assert!(compare(&a, &b, Operation::Displace { dx, dy }).is_err());
    }
}
#[test]
fn malformed_rasters_and_resource_exhaustion_are_refused() {
    let a = baseline();
    for change in 0..5 {
        let mut b = a.clone();
        match change {
            0 => {
                b.rasters[0].pixels.pop();
            }
            1 => b.rasters[0].stride = usize::MAX,
            2 => b.rasters[0].width = usize::MAX,
            3 => b.rasters[0].scale = f64::INFINITY,
            _ => b.rasters[0].channels = 1,
        }
        assert!(compare(&a, &b, Operation::Identity).is_err());
    }
    let mut large = a.clone();
    large.glyphs = vec![large.glyphs[0].clone(); 2_000];
    assert_eq!(
        compare(&large, &large, Operation::Identity).unwrap_err(),
        "Ambiguous text exceeds matching budget"
    );
}
#[test]
fn missing_targets_and_changed_graphics_are_refused() {
    let mut a = baseline();
    a.glyphs[0].target = false;
    assert!(compare_move(&a, &a).is_err());
    a.graphics.push(Graphic {
        kind: 2,
        page: 0,
        bounds: bounds(60.0, 40.0),
    });
    let mut b = a.clone();
    b.graphics[0].bounds.width += 1.0;
    assert!(compare(&a, &b, Operation::Identity).is_err());
}
