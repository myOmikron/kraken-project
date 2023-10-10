//! Some oauth primitives and the global [`OauthManager`]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

pub mod schemas;

/// A synced collection of open and accepted oauth requests
///
/// - **open request** are waiting for the user's interaction
/// - **accepted request** are waiting for the application to retrieve its token
#[derive(Debug, Default)]
pub struct OauthManager(Mutex<OauthManagerInner>);
#[derive(Debug, Default)]
struct OauthManagerInner {
    /// Waiting for user interaction i.e. `/accept` or `/deny`
    ///
    /// Uses a `uuid` as key which is presented to the user's agent
    open: HashMap<Uuid, OAuthRequest>,

    /// Waiting for server interaction i.e. `/token`
    ///
    /// Uses `code` as key which is passed through the user's agent to the client
    accepted: HashMap<Uuid, OAuthRequest>,
}

impl OauthManager {
    /// Insert an open request
    pub fn insert_open(&self, request: OAuthRequest) -> Uuid {
        let mut inner = self.0.lock().unwrap();
        loop {
            let uuid = Uuid::new_v4();
            if let Entry::Vacant(entry) = inner.open.entry(uuid) {
                entry.insert(request);
                return uuid;
            }
        }
    }

    /// Insert an accepted request
    pub fn insert_accepted(&self, request: OAuthRequest) -> Uuid {
        let mut inner = self.0.lock().unwrap();
        loop {
            let uuid = Uuid::new_v4();
            if let Entry::Vacant(entry) = inner.accepted.entry(uuid) {
                entry.insert(request);
                return uuid;
            }
        }
    }

    /// Get an open request
    pub fn get_open(&self, code: Uuid) -> Option<OAuthRequest> {
        let inner = self.0.lock().unwrap();
        inner.open.get(&code).cloned()
    }

    /// Get an accepted request
    pub fn get_accepted(&self, code: Uuid) -> Option<OAuthRequest> {
        let inner = self.0.lock().unwrap();
        inner.accepted.get(&code).cloned()
    }

    /// Remove an open request if it passes a check
    pub fn remove_open_if(
        &self,
        code: Uuid,
        check: impl FnOnce(&OAuthRequest) -> bool,
    ) -> Result<OAuthRequest, OpenIfError> {
        let mut inner = self.0.lock().unwrap();
        match inner.open.entry(code) {
            Entry::Vacant(_) => Err(OpenIfError::NotFound),
            Entry::Occupied(entry) => {
                if check(entry.get()) {
                    Ok(entry.remove())
                } else {
                    Err(OpenIfError::FailedCheck)
                }
            }
        }
    }
}

/// Error type returned by [`OauthManager::remove_open_if`]
pub enum OpenIfError {
    /// The given uuid was not found
    NotFound,
    /// The request didn't pass the check
    FailedCheck,
}

/// Open oauth request which is waiting for user interactions
#[derive(Debug, Clone)]
pub struct OAuthRequest {
    /// Pk of the requesting [`OauthClient`]
    pub client_pk: Uuid,

    /// State provided by client in `/auth`
    pub state: String,

    /// Scope requested by client
    pub scope: OAuthScope,

    /// User which is being asked
    pub user: Uuid,

    /// pkce's `code_challenge` with method `S256`
    pub code_challenge: String,
}

/// An [`OAuthRequest`]'s scope
///
/// This struct is parsed from the scope string defined by oauth.
#[derive(Debug, Copy, Clone)]
pub struct OAuthScope {
    /// A workspace's uuid
    pub workspace: Uuid,
}
