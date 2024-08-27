use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

/// The request to login
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct LoginRequest {
    /// The username that should be used for login
    #[schema(example = "user123")]
    pub username: String,
    /// The password that should be used for login
    #[schema(example = "super-secure-password")]
    pub password: String,
}

/// The request to finish the registration of a security key
#[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
pub struct FinishRegisterRequest {
    /// The public key credentials register request
    // TODO: provide a example json for this request
    #[serde(flatten)]
    #[schema(example = json!({}), value_type = Object)]
    pub register_pk_credential: RegisterPublicKeyCredential,
    /// Name of the key
    #[schema(example = "my-security-key-01")]
    pub name: String,
}
