mod cert;

use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use actix_web::web::Data;
use log::error;
use rcgen::{Certificate, CertificateParams, KeyPair, RcgenError};
use serde::Serialize;
use thiserror::Error;
use tonic::transport::{Certificate as TonicCertificate, ClientTlsConfig, ServerTlsConfig};
use url::Url;
use utoipa::ToSchema;

use crate::api::handler::ApiError;
use crate::modules::tls::cert::CertificateBuilder;

pub struct TlsManager {
    /// The CA's certificate with its private key
    ca: Certificate,

    /// The CA as expected by `tonic`
    tonic_ca: TonicCertificate,
}

#[derive(Debug, ToSchema, Serialize)]
pub struct LeechCert {
    /// PEM encoded certificate
    pub cert: String,

    /// PEM encoded private key for the certificate
    pub key: String,
}

impl TlsManager {
    pub fn load(dir: impl AsRef<Path>) -> Result<Data<Self>, TlsManagerError> {
        let dir = dir.as_ref();
        if !dir.is_dir() {
            return Err(TlsManagerError::DirNotFound(dir.to_path_buf()));
        }
        let cert_path = dir.join("ca.crt");
        let key_path = dir.join("ca.key");

        let ca;
        if !cert_path.exists() || !key_path.exists() {
            ca = cert::CA.build()?;
            fs::write(cert_path, ca.serialize_pem()?)?;
            fs::write(key_path, ca.serialize_private_key_pem())?;
        } else {
            let cert_pem = fs::read_to_string(cert_path)?;
            let key_pem = fs::read_to_string(key_path)?;
            ca = Certificate::from_params(CertificateParams::from_ca_cert_pem(
                &cert_pem,
                KeyPair::from_pem(&key_pem)?,
            )?)?;
        }

        let tonic_ca = TonicCertificate::from_pem(ca.serialize_pem()?);
        Ok(Data::new(Self { ca, tonic_ca }))
    }

    /// Generate a new certificate for a leech
    pub fn gen_leech_cert(&self) -> Result<LeechCert, TlsManagerError> {
        let cert = cert::Leech {
            url: Url::parse("https://10.13.37.11:31337").unwrap(),
        }
        .build()?;
        let cert_pem = cert.serialize_pem_with_signer(&self.ca)?;
        let key_pem = cert.serialize_private_key_pem();
        Ok(LeechCert {
            cert: cert_pem,
            key: key_pem,
        })
    }

    /// Get tonic's tls config to use when connecting to the leech
    pub fn tonic_client(&self) -> ClientTlsConfig {
        ClientTlsConfig::new().ca_certificate(self.tonic_ca.clone())
    }

    /// Get tonic's tls config to use when listening for leeches' connections
    pub fn tonic_server(&self) -> ServerTlsConfig {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum TlsManagerError {
    #[error("{0}")]
    Rcgen(#[from] RcgenError),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Directory not found: {0}")]
    DirNotFound(PathBuf),
}

impl From<TlsManagerError> for ApiError {
    fn from(value: TlsManagerError) -> Self {
        error!("tls manager error in api: {value}");
        Self::InternalServerError
    }
}

impl fmt::Debug for TlsManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TlsManager").finish_non_exhaustive()
    }
}
