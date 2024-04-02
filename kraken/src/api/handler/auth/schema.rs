use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

/// The request to login
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct LoginRequest {
    /// The username that should be used for login
    pub username: String,
    /// The password that should be used for login
    pub password: String,
}

/// The request to finish the registration of a security key
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct FinishRegisterRequest {
    /// The public key credentials register request
    // TODO: provide a example json for this request
    #[serde(flatten)]
    #[schemars(with = "serde_json::Value")]
    pub register_pk_credential: RegisterPublicKeyCredential,
    /// Name of the key
    pub name: String,
}
