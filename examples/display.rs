use raw_input::Display;

fn main() {
    test_cursor_position_mapping();
    test_monitor_logic_and_consistency();
    test_screen_size_matching();
    test_monitor_boundary_mapping();
    test_scale_factor_sanity();
    test_no_monitor_overlap();
    test_topology_connectivity();
    test_dpi_initialization_consistency();
    test_cursor_sampling_stability();
    println!("\nAll display tests passed.");
}

fn test_cursor_position_mapping() {
    let (x, y) = Display::get_cursor_position().unwrap();
    println!("\n[Test] Current Physical Position: ({}, {})", x, y);

    let monitor = Display::get_monitor_from_point(x, y);
    assert!(
        monitor.is_some(),
        "The cursor at ({}, {}) must be within the bounds of a connected monitor.",
        x,
        y
    );
    println!("  PASS");
}

fn test_monitor_logic_and_consistency() {
    let monitors = Display::get_available_monitors();
    assert!(!monitors.is_empty(), "At least one monitor should be detected.");

    let primary_monitors: Vec<_> = monitors.iter().filter(|m| m.is_primary).collect();
    assert_eq!(
        primary_monitors.len(),
        1,
        "System must have exactly one primary monitor. Found: {}",
        primary_monitors.len()
    );

    let primary = Display::get_primary_monitor().expect("Primary monitor helper failed.");
    assert!(primary.is_primary);
    assert_eq!(
        primary.offset,
        (0.0, 0.0),
        "The primary monitor origin should typically be (0, 0)."
    );
    println!("  PASS");
}

fn test_screen_size_matching() {
    let (sw, sh) = Display::get_primary_screen_size();
    if let Some(primary) = Display::get_primary_monitor() {
        assert_eq!(
            (sw, sh),
            primary.size,
            "Global screen size metrics must match the primary monitor's physical size."
        );
    }
    println!("  PASS");
}

fn test_monitor_boundary_mapping() {
    let monitors = Display::get_available_monitors();

    for m in monitors {
        let corners = [
            (m.offset.0, m.offset.1),
            (m.offset.0 + m.size.0 - 1.0, m.offset.1),
            (m.offset.0, m.offset.1 + m.size.1 - 1.0),
            (m.offset.0 + m.size.0 - 1.0, m.offset.1 + m.size.1 - 1.0),
        ];

        for (cx, cy) in corners {
            let found = Display::get_monitor_from_point(cx, cy);
            assert!(
                found.is_some(),
                "Point ({}, {}) should be inside monitor '{}'",
                cx,
                cy,
                m.name
            );
            assert_eq!(found.unwrap().name, m.name);
        }
    }
    println!("  PASS");
}

fn test_scale_factor_sanity() {
    let monitors = Display::get_available_monitors();
    for m in monitors {
        assert!(
            m.scale_factor >= 1.0 && m.scale_factor <= 5.0,
            "Monitor '{}' has an unusual scale factor: {:.2}",
            m.name,
            m.scale_factor
        );
    }
    println!("  PASS");
}

fn test_no_monitor_overlap() {
    let monitors = Display::get_available_monitors();
    for (i, m1) in monitors.iter().enumerate() {
        for m2 in monitors.iter().skip(i + 1) {
            let overlaps = !(
                m1.offset.0 + m1.size.0 <= m2.offset.0
                    || m2.offset.0 + m2.size.0 <= m1.offset.0
                    || m1.offset.1 + m1.size.1 <= m2.offset.1
                    || m2.offset.1 + m2.size.1 <= m1.offset.1
            );
            assert!(
                !overlaps,
                "Detected overlapping monitors: '{}' and '{}'",
                m1.name, m2.name
            );
        }
    }
    println!("  PASS");
}

fn test_topology_connectivity() {
    let monitors = Display::get_available_monitors();
    if monitors.len() < 2 {
        println!("  SKIP (single monitor)");
        return;
    }

    for m in &monitors {
        let probe_points = [
            (m.offset.0 - 1.0, m.offset.1),
            (m.offset.0 + m.size.0, m.offset.1),
            (m.offset.0, m.offset.1 - 1.0),
            (m.offset.0, m.offset.1 + m.size.1),
        ];

        for (px, py) in probe_points {
            let found = Display::get_monitor_from_point(px, py);
            if let Some(other) = found {
                assert_ne!(
                    m.name, other.name,
                    "Point ({}, {}) outside '{}' should not map back to itself.",
                    px, py, m.name
                );
            }
        }
    }
    println!("  PASS");
}

fn test_dpi_initialization_consistency() {
    let initial_scale = Display::get_scale_factor();
    for _ in 0..10 {
        assert_eq!(
            initial_scale,
            Display::get_scale_factor(),
            "Scale factor changed unexpectedly between calls. DPI awareness might be unstable."
        );
    }
    println!("  PASS");
}

fn test_cursor_sampling_stability() {
    let mut last_pos = Display::get_cursor_position().unwrap();
    for _ in 0..50 {
        let current_pos = Display::get_cursor_position().unwrap();
        let delta_x = (current_pos.0 - last_pos.0).abs();
        let delta_y = (current_pos.1 - last_pos.1).abs();

        assert!(
            delta_x < 500.0 && delta_y < 500.0,
            "Large cursor jump detected. DPI awareness may be unstable."
        );
        last_pos = current_pos;
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    println!("  PASS");
}
