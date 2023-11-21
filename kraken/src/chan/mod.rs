//! All channels that are used throughout kraken

use std::ops::Deref;
use std::sync::{Arc, OnceLock, RwLock};

pub use dehashed_manager::*;
use dehashed_rs::Scheduler;
pub use leech_manager::*;
use rorm::Database;
pub use settings_manager::*;
pub use ws_manager::*;

use crate::modules::tls::TlsManager;

mod dehashed_manager;
mod health_manager;
mod leech_manager;
mod settings_manager;
mod ws_manager;

/// Set of global managers and handles
pub static GLOBAL: GlobalOnceCell<GlobalChan> = GlobalOnceCell::new();

/// Set of global managers and handles
pub struct GlobalChan {
    /// The database
    pub db: Database,

    /// Collection of all connected leeches
    pub leeches: Arc<LeechManager>,

    /// Collection of all connected websockets
    pub ws: WsManagerChan,

    /// Settings which may change without restarting the server
    pub settings: Arc<SettingsManagerChan>,

    /// Scheduler for performing rate-limited dehashed requests
    pub dehashed: Arc<RwLock<Option<Scheduler>>>,

    /// Kraken's CA and certificate
    pub tls: Arc<TlsManager>,
}

/// Simple [`OnceLock`] which panics in case of error.
pub struct GlobalOnceCell<T>(OnceLock<T>);
impl<T> GlobalOnceCell<T> {
    /// Creates a new empty cell
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    /// Initialise the cell
    ///
    /// ## Panics
    /// If called twice
    pub fn init(&self, value: T) {
        self.0
            .set(value)
            .ok()
            .expect("`GlobalLock.init` has been called twice")
    }
}
impl<T> Deref for GlobalOnceCell<T> {
    type Target = T;

    /// Retrieved the initialised value
    ///
    /// ## Panics
    /// If called before [`GlobalOnceCell::init`]
    fn deref(&self) -> &Self::Target {
        self.0
            .get()
            .expect("`GlobalLock.init` has not been called yet")
    }
}
