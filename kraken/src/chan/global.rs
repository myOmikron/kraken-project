//! Set of global managers and handles

use std::ops::Deref;
use std::sync::{Arc, OnceLock, RwLock};

use dehashed_rs::Scheduler;
use rorm::Database;

use crate::chan::leech_manager::LeechManager;
use crate::chan::settings_manager::SettingsManagerChan;
use crate::chan::ws_manager::chan::WsManagerChan;
use crate::modules::aggregator::Aggregator;
use crate::modules::cache::{UserCache, WorkspaceCache};
use crate::modules::editor::EditorSync;
use crate::modules::tls::TlsManager;

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
    pub settings: SettingsManagerChan,

    /// Scheduler for performing rate-limited dehashed requests
    pub dehashed: RwLock<Option<Scheduler>>,

    /// Kraken's CA and certificate
    ///
    /// It is is wrapped by an `Arc` as we may want to make future changes in `TlsManager`
    /// that would required the `Arc`
    pub tls: Arc<TlsManager>,

    /// The caching layer for workspace members
    pub workspace_cache: WorkspaceCache,

    /// The caching layer for users
    pub user_cache: UserCache,

    /// Scheduler for inserting or updating any aggregation model in the database
    pub aggregator: Aggregator,

    /// Live synchronization for the editor in the frontend
    pub editor_sync: EditorSync,
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
        #[allow(clippy::expect_used)]
        self.0
            .get()
            .expect("`GlobalLock.init` has not been called yet. Please open an issues.")
    }
}
