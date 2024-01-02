use tui_input::Input;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use crate::model::client::Client;

pub enum Screen {
    Login,
    Client,
}

pub enum Popup {
    LoginSuccessful,
}

pub enum InputMode {
    Normal,
    /// The value represents the InputField being edited
    Editing(u8),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TimeoutType {
    Resize,
    Login,
}

pub struct InputFields(pub Input, pub Input);

pub struct Timer {
    pub counter: u8,
    pub tick_rate: Duration,
    pub last_update: Instant,
}

pub struct App {
    pub input: InputFields,
    pub input_mode: InputMode,
    pub failed_logins: u8,
    pub active_user: Option<Client>,
    pub timeout: HashMap<TimeoutType, Timer>,
    pub curr_screen: Screen,
    pub active_popup: Option<Popup>,
    pub should_clear_screen: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            input: InputFields(Input::default(), Input::default()),
            input_mode: InputMode::Normal,
            failed_logins: 0,
            active_user: None,
            timeout: HashMap::new(),
            curr_screen: Screen::Login,
            active_popup: None,
            should_clear_screen: false,
            should_quit: false,
        }
    }
 
    pub fn enter_screen(&mut self, screen: Screen) {
        self.should_clear_screen = true;
        match screen {
            Screen::Login => {
                self.curr_screen = Screen::Login;
                self.input_mode = InputMode::Editing(0);
                self.failed_logins = 0;
                self.active_user = None;
                self.input.0.reset();
                self.input.1.reset();
            }
            Screen::Client => {
                self.curr_screen = Screen::Client;
                self.input_mode = InputMode::Normal;
            }
            _ => { unimplemented!() }
        }
    }

    /// The timeout tick rate here should be equal or greater to the EventHandler tick rate.
    /// This is important because the minimum update time perceivable is defined by the EventHandler tick rate.
    pub fn add_timeout(&mut self, counter: u8, tick_rate: u16, timeout_type: TimeoutType) {
        if self.timeout.contains_key(&timeout_type) {
            panic!("cannot add timeout {:?} to list of timeouts. It already exists", timeout_type);
        }

        let tick_rate = Duration::from_millis(tick_rate as u64);

        self.timeout.insert(timeout_type, Timer{
            counter,
            tick_rate,
            last_update: Instant::now(),
        });
    }

    pub fn update_timeout_counter(&mut self, timeout_type: TimeoutType) {
        let timer = self.timeout.get_mut(&timeout_type)
            .unwrap_or_else(|| panic!("tried to update a nonexistent timeout"));

        if timer.counter > 1 {
            timer.counter -= 1;
            timer.last_update = Instant::now();
        } else {
            match timeout_type {
                TimeoutType::Login => self.failed_logins = 0,
                _ => {}
            }
            self.timeout.remove(&timeout_type);
        }
    }
}