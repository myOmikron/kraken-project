use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs, io};

use actix_web::web::Data;
use log::error;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyPair, KeyUsagePurpose, RcgenError, SanType, PKCS_ECDSA_P256_SHA256,
};
use serde::Serialize;
use thiserror::Error;
use tonic::transport::{Certificate as TonicCertificate, ClientTlsConfig, ServerTlsConfig};
use utoipa::ToSchema;

use crate::api::handler::ApiError;

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
            ca = new_cert(ca_params)?;
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
        let cert = new_cert(leech_params)?;
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

fn new_cert(builder: impl FnOnce(&mut CertificateParams)) -> Result<Certificate, RcgenError> {
    let mut params = CertificateParams::default();
    builder(&mut params);
    Certificate::from_params(params)
}
fn ca_params(params: &mut CertificateParams) {
    params.alg = &PKCS_ECDSA_P256_SHA256;
    params
        .distinguished_name
        .push(DnType::CommonName, "kraken CA");
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
}
fn leech_params(params: &mut CertificateParams) {
    params.alg = &PKCS_ECDSA_P256_SHA256;
    params
        .distinguished_name
        .push(DnType::CommonName, "leech cert");
    params.is_ca = IsCa::ExplicitNoCa;
    params.key_usages.extend([
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ]);
    params
        .extended_key_usages
        .extend([ExtendedKeyUsagePurpose::ServerAuth]);
    params
        .subject_alt_names
        .extend([SanType::IpAddress(IpAddr::from_str("10.13.37.11").unwrap())]); // TODO parse alt name from address
    params.use_authority_key_identifier_extension = true;
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
