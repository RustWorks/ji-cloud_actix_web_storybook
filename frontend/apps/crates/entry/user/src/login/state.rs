use dominator_helpers::futures::AsyncLoader;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use std::cell::RefCell;

pub struct State {
    pub loader: AsyncLoader,
    pub email: RefCell<String>,
    pub password: RefCell<String>,
    pub status: Mutable<Option<Status>>,
}
impl State {
    pub fn new() -> Self {
        Self {
            loader: AsyncLoader::new(),
            email: RefCell::new("".to_string()),
            password: RefCell::new("".to_string()),
            status: Mutable::new(None),
        }
    }

    pub fn clear_email_status(&self) {
        if self
            .status
            .get_cloned()
            .and_then(|x| x.email_error())
            .is_some()
        {
            self.status.set(None);
        }
    }
    pub fn clear_password_status(&self) {
        if self
            .status
            .get_cloned()
            .and_then(|x| x.password_error())
            .is_some()
        {
            self.status.set(None);
        }
    }

    pub fn email_error(&self) -> impl Signal<Item = &'static str> {
        self.status
            .signal_cloned()
            .map(|err| err.and_then(|err| err.email_error()).unwrap_or(""))
    }

    pub fn password_error(&self) -> impl Signal<Item = &'static str> {
        self.status
            .signal_cloned()
            .map(|err| err.and_then(|err| err.password_error()).unwrap_or(""))
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    BadCredentials,
    PasswordResetSent,
}

impl Status {
    pub fn email_error(&self) -> Option<&'static str> {
        match self {
            Self::PasswordResetSent => Some("Check your email at this address!"),
            Self::BadCredentials => Some("Invalid email or password"),
        }
    }
    pub fn password_error(&self) -> Option<&'static str> {
        match self {
            Self::PasswordResetSent => Some("Password reset link sent!"),
            Self::BadCredentials => Some("Invalid email or password"),
        }
    }
}
