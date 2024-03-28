use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

/// The request to login
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct LoginRequest {
    /// The username that should be used for login
    // TODO #[schema(example = "user123")]
    pub username: String,
    /// The password that should be used for login
    // TODO #[schema(example = "super-secure-password")]
    pub password: String,
}

/// The request to finish the registration of a security key
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct FinishRegisterRequest {
    /// The public key credentials register request
    // TODO: provide a example json for this request
    #[serde(flatten)]
    // TODO #[schema(example = json!({}), value_type = Object)]
    #[schemars(with = "serde_json::Value")]
    pub register_pk_credential: RegisterPublicKeyCredential,
    /// Name of the key
    // TODO #[schema(example = "my-security-key-01")]
    pub name: String,
}
