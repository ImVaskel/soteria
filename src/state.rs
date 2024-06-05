use eyre::Result;
use gtk4::{
    glib::{clone, SignalHandlerId},
    prelude::*,
    ApplicationWindow, Button, DropDown, PasswordEntry, StringObject,
};
use tokio::sync::broadcast;

use crate::events::AuthenticationEvent;

#[derive(Debug)]
struct ComponentSignals {
    window_close: SignalHandlerId,
    cancel_button_clicked: SignalHandlerId,
    confirm_button_clicked: SignalHandlerId,
}

#[derive(Debug, Clone)]
struct Components {
    cancel_button: Button,
    confirm_button: Button,
    password_entry: PasswordEntry,
    window: ApplicationWindow,
    dropdown: DropDown,
}

/// Holds the state for the application
/// This just holds the current cookie being used for authentication and the handlers associated with it.
/// The methods ``start_authentication`` and ``finish_authentication`` respectively should be called so that
/// the signal handlers can be correctly added/removed.
#[derive(Debug)]
pub struct State {
    cookie: Option<String>,
    signals: Option<ComponentSignals>,
    sender: broadcast::Sender<AuthenticationEvent>,
    components: Components,
}

impl State {
    pub fn new(
        sender: broadcast::Sender<AuthenticationEvent>,
        cancel_button: Button,
        confirm_button: Button,
        password_entry: PasswordEntry,
        window: ApplicationWindow,
        dropdown: DropDown,
    ) -> Self {
        Self {
            cookie: None,
            signals: None,
            sender,
            components: Components {
                cancel_button,
                confirm_button,
                password_entry,
                window,
                dropdown,
            },
        }
    }

    pub fn start_authentication(&mut self, cookie: String) -> Result<bool> {
        // if self.cookie.is_some() {
        //     tracing::debug!("recieved request to authenticate and we are already running with cookie {:?}", self.cookie);
        //     self.sender.send(AuthenticationEvent::AlreadyRunning {
        //         cookie: cookie.clone(),
        //     })?;
        //     return Ok(false);
        // }

        self.cookie = Some(cookie);

        let window_close = self.components.window.connect_hide_on_close_notify(clone!(@strong self.sender as sender, @strong self.components as components, @strong self.cookie as cookie => move |_| {
            sender.send(AuthenticationEvent::UserCanceled{cookie: cookie.clone().unwrap()}).unwrap();
            components.password_entry.set_text("");
        }));

        let cancel_button_clicked = self.components.cancel_button.connect_clicked(clone!(@strong self.sender as sender, @strong self.components as components, @strong self.cookie as cookie=> move |_| {
            sender.send(AuthenticationEvent::UserCanceled{cookie: cookie.clone().unwrap()}).unwrap();
            components.password_entry.set_text("");
            components.window.set_visible(false);
        }));

        let confirm_button_clicked = self.components.confirm_button.connect_clicked(clone!(@strong self.sender as sender, @strong self.components as components, @strong self.cookie as cookie => move |_| {
            let pw = components.password_entry.text();
            let user: StringObject = components.dropdown.selected_item().unwrap().dynamic_cast().unwrap();
            sender.send(AuthenticationEvent::UserProvidedPassword { cookie: cookie.clone().unwrap(), username: user.string().to_string(), password: pw.to_string()}).unwrap();
            components.password_entry.set_text("");
            components.window.set_visible(false);
        }));

        self.signals = Some(ComponentSignals {
            window_close,
            cancel_button_clicked,
            confirm_button_clicked,
        });

        Ok(true)
    }

    pub fn end_authentication(&mut self, cookie: &str) {
        if let Some(c) = &self.cookie {
            if c == cookie {
                self.components.window.set_visible(false);
                if let Some(ComponentSignals {
                    window_close,
                    cancel_button_clicked,
                    confirm_button_clicked,
                }) = self.signals.take()
                {
                    self.components
                        .cancel_button
                        .disconnect(cancel_button_clicked);
                    self.components
                        .confirm_button
                        .disconnect(confirm_button_clicked);
                    self.components.window.disconnect(window_close);
                }
                self.cookie = None;
            }
        }
    }
}
