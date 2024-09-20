//! This module provides a [`Tui`] wrapper, implementing standard entry and
//! exit procedures to prepare rendering on the terminal.

// TODO: Convert this module to use color_eyre instead of panic and hook

use std::{
    io::{stderr, Stderr},
    ops::{Deref, DerefMut},
    panic::set_hook,
    process::exit,
};

use color_eyre::{config::HookBuilder, Result};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend as Backend, Terminal};
use tracing::error;

/// [`Tui`] is a thin wrapper over [`ratatui`] with [`crossterm`] backend
/// providing methods to handle terminal based operations.
#[derive(Debug)]
pub struct Tui {
    /// Crossterm backend terminal instance.
    pub(self) terminal: Terminal<Backend<Stderr>>,
}

// TODO: Use wrap_err wherever context needs to be added

impl Tui {
    /// Creates new Tui instance with crossterm backend.
    #[inline]
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: Terminal::new(Backend::new(stderr()))?,
        })
    }

    /// Activates Tui instance and registers panic handlers.
    pub fn enter(&self) -> Result<()> {
        enable_raw_mode()?;
        init_panic_hook()?;
        crossterm::execute!(
            stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(())
    }

    /// Deactivates Tui instance.
    pub fn exit(&self) -> Result<()> {
        crossterm::execute!(
            stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        disable_raw_mode()?;
        Ok(())
    }

    /// Suspends Tui instance.
    pub fn suspend(&self) -> Result<()> {
        self.exit()?;
        #[expect(
            unsafe_code,
            reason = "Intended: Instead of directly exiting, this allows terminal to cleanup and return back to normal mode."
        )]
        #[cfg(not(windows))]
        // SAFETY: Sending terminal stop signal marks the end of the terminal access operations.
        // There should be no operation sent after this point.
        unsafe {
            _ = libc::raise(libc::SIGTSTP);
        }
        Ok(())
    }

    /// Resumes Tui instance.
    pub fn resume(&self) -> Result<()> {
        self.enter()?;
        Ok(())
    }
}

impl Deref for Tui {
    type Target = Terminal<Backend<Stderr>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    #[inline]
    fn drop(&mut self) {
        assert!(
            self.exit().is_ok(),
            "Failed to drop tui as exit call failed."
        );
    }
}

/// Installs custom panic hook to work with `color_eyre`, `human_panic` and
/// `better_panic`.
fn init_panic_hook() -> Result<()> {
    // TODO: Use this reporting text with color_eyre as well
    let (panic_hook, eyre_hook) = HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}.",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .display_location_section(true)
        .display_env_section(true)
        .into_hooks();

    eyre_hook.install()?;

    set_hook(Box::new(move |panic_info| {
        if let Ok(tui) = Tui::new() {
            if let Err(err) = tui.exit() {
                error!("Unable to exit Terminal: {:?}.", err);
            }
        }

        let msg = format!("{}", panic_hook.panic_report(panic_info));

        #[cfg(not(debug_assertions))]
        {
            eprintln!("{}", msg); // prints color-eyre stack trace to stderr
            use human_panic::{handle_dump, print_msg, Metadata};
            let meta = Metadata {
                version: env!("CARGO_PKG_VERSION").into(),
                name: env!("CARGO_PKG_NAME").into(),
                authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
                homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            };

            let file_path = handle_dump(&meta, panic_info);
            // prints human-panic message
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }

        error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            use better_panic::{Settings, Verbosity};
            // Better Panic stacktrace that is only enabled when debugging.
            Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        #[expect(
            clippy::exit,
            reason = "Intended: Calling exit inside panic hook is acceptable."
        )]
        exit(libc::EXIT_FAILURE);
    }));

    Ok(())
}
