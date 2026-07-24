use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};
use tokio::sync::mpsc;

/// Events produced by the event loop and consumed by the main application.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    /// A key press event.
    Key(KeyEvent),
    /// Terminal resize event.
    Resize(u16, u16),
    /// Periodic tick (for UI updates like loading spinners).
    Tick,
}

/// Async event handler that bridges crossterm events into a tokio channel.
///
/// Runs a background task that polls for terminal events and sends them
/// through an unbounded channel, keeping the main async loop responsive.
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate.
    ///
    /// The tick rate determines how often `Tick` events are emitted when
    /// no terminal events are available.
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let task = tokio::task::spawn_blocking(move || {
            loop {
                // Poll for events with the tick rate as timeout
                let has_event = event::poll(tick_rate).unwrap_or(false);

                if has_event {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            // Only process Press events to avoid duplicates
                            if key.kind == crossterm::event::KeyEventKind::Press {
                                if tx.send(AppEvent::Key(key)).is_err() {
                                    return; // Channel closed, receiver dropped
                                }
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if tx.send(AppEvent::Resize(w, h)).is_err() {
                                return;
                            }
                        }
                        Ok(_) => {} // Ignore mouse, focus, paste events
                        Err(_) => return,
                    }
                } else {
                    // No event within tick_rate — send a tick
                    if tx.send(AppEvent::Tick).is_err() {
                        return;
                    }
                }
            }
        });

        Self { rx, _task: task }
    }

    /// Wait for the next event from the channel.
    pub async fn next(&mut self) -> anyhow::Result<AppEvent> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
