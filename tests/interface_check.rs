#[cfg(test)]
mod interface_checks {
    use raw_input::{Core, CoreError, Display, Event, Grab, Key, Listen, MouseButton, Simulate};

    #[test]
    fn check_core() {
        let _: fn() -> Result<(), CoreError> = Core::start;
        let _: fn() -> bool = Core::is_runing;
        let _: fn() = Core::pause;
        let _: fn() = Core::resume;
        let _: fn() = Core::stop;
    }

    #[test]
    fn check_listen() {
        let _: fn() = Listen::start;
        let _: fn() -> bool = Grab::is_runing;
        let _: fn() = Listen::pause;
        let _: fn() = Listen::resume;
        let _: fn() = Listen::stop;
        let _: fn(bool) = Listen::mouse_move;
        let _: fn(bool) = Listen::mouse_wheel;
        let _: fn(bool) = Listen::mouse_button;
        let _: fn(bool) = Listen::keyboard;

        fn _check_subscribe<F>(f: F)
        where
            F: Fn(Event) + Send + Sync + 'static,
        {
            let _: raw_input::SubscriptionHandle = Listen::subscribe(f);
        }
        let _: fn() = Listen::unsubscribe_all;
    }

    #[test]
    fn check_grab() {
        let _: fn() = Grab::start;
        let _: fn() -> bool = Grab::is_runing;
        let _: fn() = Grab::pause;
        let _: fn() = Grab::resume;
        let _: fn() = Grab::stop;
        let _: fn(bool) = Grab::mouse_move;
        let _: fn(bool) = Grab::mouse_wheel;
        let _: fn(bool) = Grab::mouse_button;
        let _: fn(bool) = Grab::keyboard;
    }

    #[test]
    fn check_display() {
        let _: fn() -> f64 = Display::get_scale_factor;
        let _: fn() -> Option<(f64, f64)> = Display::get_cursor_position;
        let _: fn() -> (f64, f64) = Display::get_primary_screen_size;
        let _: fn() -> (f64, f64) = Display::get_virtual_screen_size;
        let _: fn() -> (f64, f64, f64, f64) = Display::get_virtual_screen_bounds;
        let _: fn() -> Vec<raw_input::MonitorInfo> = Display::get_available_monitors;
        let _: fn() -> Option<raw_input::MonitorInfo> = Display::get_primary_monitor;
        let _: fn() -> Option<raw_input::MonitorInfo> = Display::get_current_monitor;
        let _: fn(f64, f64) -> Option<raw_input::MonitorInfo> = Display::get_monitor_from_point;
    }

    #[test]
    fn check_simulate() {
        let _: fn(Event) = Simulate::simulate;
        let _: fn(f64, f64) = Simulate::mouse_move;
        let _: fn(f64, f64) = Simulate::mouse_move_to;
        let _: fn(f64, f64) = Simulate::mouse_wheel;
        let _: fn(MouseButton, bool) = Simulate::mouse_button;
        let _: fn(Key, bool) = Simulate::keyboard;
    }
}
