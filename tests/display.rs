#[cfg(test)]
mod display_tests {
    use raw_input::Display;

    /// Verifies that the physical cursor position can always be mapped
    /// to at least one connected monitor.
    #[test]
    fn test_cursor_position_mapping() {
        let (x, y) = Display::get_cursor_position();
        println!("\n[Test] Current Physical Position: ({}, {})", x, y);

        let monitor = Display::get_monitor_from_point(x, y);
        assert!(
            monitor.is_some(),
            "The cursor at ({}, {}) must be within the bounds of a connected monitor.",
            x,
            y
        );
    }

    /// Checks the structural integrity of the monitor list.
    /// Ensures there is exactly one primary monitor and no coordinate overlaps.
    #[test]
    fn test_monitor_logic_and_consistency() {
        let monitors = Display::get_available_monitors();
        assert!(
            !monitors.is_empty(),
            "At least one monitor should be detected."
        );

        let primary_monitors: Vec<_> = monitors.iter().filter(|m| m.is_primary).collect();

        // Strict Constraint: Windows always designates exactly one primary monitor.
        assert_eq!(
            primary_monitors.len(),
            1,
            "System must have exactly one primary monitor. Found: {}",
            primary_monitors.len()
        );

        // Verify that the helper method returns the same primary monitor.
        let primary = Display::get_primary_monitor().expect("Primary monitor helper failed.");
        assert!(primary.is_primary);
        assert_eq!(
            primary.offset,
            (0, 0),
            "The primary monitor origin should typically be (0, 0)."
        );
    }

    /// Validates that the global screen size matches the primary monitor's size.
    #[test]
    fn test_screen_size_matching() {
        let (sw, sh) = Display::get_primary_screen_size();
        if let Some(primary) = Display::get_primary_monitor() {
            assert_eq!(
                (sw, sh),
                primary.size,
                "Global screen size metrics must match the primary monitor's physical size."
            );
        }
    }

    /// Tests the boundaries of each monitor to ensure coordinate logic is inclusive/exclusive correctly.
    #[test]
    fn test_monitor_boundary_mapping() {
        let monitors = Display::get_available_monitors();

        for m in monitors {
            // Test the 4 corners of the monitor rectangle
            let corners = [
                (m.offset.0, m.offset.1),                               // Top-Left
                (m.offset.0 + m.size.0 - 1, m.offset.1),                // Top-Right
                (m.offset.0, m.offset.1 + m.size.1 - 1),                // Bottom-Left
                (m.offset.0 + m.size.0 - 1, m.offset.1 + m.size.1 - 1), // Bottom-Right
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
    }

    /// Ensures scale factors are within a realistic range (100% to 500%).
    #[test]
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
    }

    /// Ensures that no two monitors have overlapping physical areas.
    #[test]
    fn test_no_monitor_overlap() {
        let monitors = Display::get_available_monitors();
        for (i, m1) in monitors.iter().enumerate() {
            for m2 in monitors.iter().skip(i + 1) {
                let overlaps = !(
                    m1.offset.0 + m1.size.0 <= m2.offset.0 || // m1 is to the left of m2
                    m2.offset.0 + m2.size.0 <= m1.offset.0 || // m2 is to the left of m1
                    m1.offset.1 + m1.size.1 <= m2.offset.1 || // m1 is above m2
                    m2.offset.1 + m2.size.1 <= m1.offset.1
                    // m2 is above m1
                );
                assert!(
                    !overlaps,
                    "Detected overlapping monitors: '{}' and '{}'",
                    m1.name, m2.name
                );
            }
        }
    }

    /// Tests if moving 1 pixel outside a non-primary monitor leads to another monitor
    /// or outside the valid virtual desktop.
    #[test]
    fn test_topology_connectivity() {
        let monitors = Display::get_available_monitors();
        if monitors.len() < 2 {
            return;
        }

        for m in &monitors {
            // Check points 1 pixel outside each edge
            let probe_points = [
                (m.offset.0 - 1, m.offset.1),        // Left
                (m.offset.0 + m.size.0, m.offset.1), // Right
                (m.offset.0, m.offset.1 - 1),        // Top
                (m.offset.0, m.offset.1 + m.size.1), // Bottom
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
    }

    #[test]
    fn test_dpi_initialization_consistency() {
        let initial_scale = Display::get_scale_factor();
        for _ in 0..10 {
            assert_eq!(
                initial_scale,
                Display::get_scale_factor(),
                "Scale factor changed unexpectedly between calls. DPI awareness might be unstable."
            );
        }
    }

    /// Ensures that physical cursor sampling is stable and doesn't jitter
    /// due to DPI awareness re-initialization.
    #[test]
    fn test_cursor_sampling_stability() {
        let mut last_pos = Display::get_cursor_position();
        for _ in 0..50 {
            let current_pos = Display::get_cursor_position();
            // Note: If the user moves the mouse during test, this is fine.
            // We are checking for massive jumps (e.g., 1.25x or 1.5x scaling differences).
            let delta_x = (current_pos.0 - last_pos.0).abs();
            let delta_y = (current_pos.1 - last_pos.1).abs();

            assert!(
                delta_x < 500 && delta_y < 500,
                "Large cursor jump detected. DPI awareness may be unstable."
            );
            last_pos = current_pos;
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
