//! leech module for parsing TXT entries in the DNS results specifically.

use std::fmt::Display;

use log::{debug, info};
use once_cell::sync::Lazy;
use regex::bytes::Regex;
use tokio::{sync::mpsc::Sender, task::JoinSet};
use trust_dns_resolver::{
    name_server::{GenericConnector, TokioRuntimeProvider},
    proto::rr::Record,
    AsyncResolver, TokioAsyncResolver,
};

use crate::modules::dns::resolve;

type ResolverT = AsyncResolver<GenericConnector<TokioRuntimeProvider>>;

use super::{
    errors::DnsResolutionError,
    spf::{parse_spf, SPFPart},
};

/// DNS TXT scanning settings
pub struct DnsTxtScanSettings {
    /// The domains to start resolving TXT settings in
    pub domains: Vec<String>,
}

/// Represents a single parsed DNS TXT entry.
#[derive(Debug, Clone)]
pub enum TxtScanInfo {
    /// regex: /^GOOGLE-SITE-VERIFICATION=/i
    /// Google Search Console
    HasGoogleAccount,
    /// regex: /globalsign/i
    /// Globalsign TLS certificate
    HasGlobalsignAccount,
    /// regex: /globalsign-smime/i
    /// Globalsign mails?
    HasGlobalsignSMime,
    /// regex: /^docusign/i
    /// DocuSign Identity Provider -> When you claim and verify an email domain for your organization, you can manage all users for that domain, across all accounts linked to the organization.
    HasDocusignAccount,
    /// regex: /^apple-domain-verification=/i
    /// owns apple account
    HasAppleAccount,
    /// regex: /^facebook-domain-verification=/i
    /// owns facebook account
    HasFacebookAccount,
    /// regex: /^hubspot-developer-verification=/i
    /// owns hubspot account (marketing tools)
    HasHubspotAccount,
    /// regex: /^d365mktkey=/i
    /// has Microsoft ERP: Dynamics 365
    HasMsDynamics365,
    /// regex: /^stripe-verification=/i
    /// uses stripe payments
    HasStripeAccount,
    /// regex: /^onetrust-domain-verification=/i
    /// might use OneTrust SSO?
    HasOneTrustSso,
    /// regex: /^brevo-code:/i
    /// Emails sent from Brevo (CRM / marketing tools)
    HasBrevoAccount,
    /// regex: /^atlassian-domain-verification=/i
    /// owns atlassian account
    OwnsAtlassianAccounts,
    /// regex: /^ZOOM_verify_/i
    /// Probably has Zoom users with emails with this domain
    OwnsZoomAccounts,
    /// regex: /^protonmail-verification=/i
    /// Emails hosted at ProtonMail
    EmailProtonMail,
    /// /^v=spf1/ and parsed SPF domains & IPs
    SPF {
        /// A list of all successfully parsed SPF parts (unparsable parts simply skipped)
        parts: Vec<SPFPart>,
    },
}

static BASIC_TXT_TYPES_WITH_REGEX: [TxtScanInfo; 14] = [
    TxtScanInfo::HasGoogleAccount,
    TxtScanInfo::HasGlobalsignAccount,
    TxtScanInfo::HasGlobalsignSMime,
    TxtScanInfo::HasDocusignAccount,
    TxtScanInfo::HasAppleAccount,
    TxtScanInfo::HasFacebookAccount,
    TxtScanInfo::HasHubspotAccount,
    TxtScanInfo::HasMsDynamics365,
    TxtScanInfo::HasStripeAccount,
    TxtScanInfo::HasOneTrustSso,
    TxtScanInfo::HasBrevoAccount,
    TxtScanInfo::OwnsAtlassianAccounts,
    TxtScanInfo::OwnsZoomAccounts,
    TxtScanInfo::EmailProtonMail,
];

impl TxtScanInfo {
    fn matcher_regex(&self) -> Option<&'static Regex> {
        static RE_HAS_GOOGLE_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^GOOGLE-SITE-VERIFICATION=").unwrap());
        static RE_HAS_GLOBALSIGN_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)globalsign").unwrap());
        static RE_HAS_GLOBALSIGN_SMIME: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)globalsign-smime").unwrap());
        static RE_HAS_DOCUSIGN_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^docusign").unwrap());
        static RE_HAS_APPLE_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^apple-domain-verification=").unwrap());
        static RE_HAS_FACEBOOK_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^facebook-domain-verification=").unwrap());
        static RE_HAS_HUBSPOT_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^hubspot-developer-verification=").unwrap());
        static RE_HAS_MS_DYNAMICS365: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^d365mktkey=").unwrap());
        static RE_HAS_STRIPE_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^stripe-verification=").unwrap());
        static RE_HAS_ONE_TRUST_SSO: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^onetrust-domain-verification=").unwrap());
        static RE_HAS_BREVO_ACCOUNT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^brevo-code:").unwrap());
        static RE_OWNS_ATLASSIAN_ACCOUNTS: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^atlassian-domain-verification=").unwrap());
        static RE_OWNS_ZOOM_ACCOUNTS: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^ZOOM_verify_").unwrap());
        static RE_EMAIL_PROTON_MAIL: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^protonmail-verification=").unwrap());

        match self {
            TxtScanInfo::HasGoogleAccount => Some(&RE_HAS_GOOGLE_ACCOUNT),
            TxtScanInfo::HasGlobalsignAccount => Some(&RE_HAS_GLOBALSIGN_ACCOUNT),
            TxtScanInfo::HasGlobalsignSMime => Some(&RE_HAS_GLOBALSIGN_SMIME),
            TxtScanInfo::HasDocusignAccount => Some(&RE_HAS_DOCUSIGN_ACCOUNT),
            TxtScanInfo::HasAppleAccount => Some(&RE_HAS_APPLE_ACCOUNT),
            TxtScanInfo::HasFacebookAccount => Some(&RE_HAS_FACEBOOK_ACCOUNT),
            TxtScanInfo::HasHubspotAccount => Some(&RE_HAS_HUBSPOT_ACCOUNT),
            TxtScanInfo::HasMsDynamics365 => Some(&RE_HAS_MS_DYNAMICS365),
            TxtScanInfo::HasStripeAccount => Some(&RE_HAS_STRIPE_ACCOUNT),
            TxtScanInfo::HasOneTrustSso => Some(&RE_HAS_ONE_TRUST_SSO),
            TxtScanInfo::HasBrevoAccount => Some(&RE_HAS_BREVO_ACCOUNT),
            TxtScanInfo::OwnsAtlassianAccounts => Some(&RE_OWNS_ATLASSIAN_ACCOUNTS),
            TxtScanInfo::OwnsZoomAccounts => Some(&RE_OWNS_ZOOM_ACCOUNTS),
            TxtScanInfo::EmailProtonMail => Some(&RE_EMAIL_PROTON_MAIL),
            _ => None,
        }
    }
}

impl Display for TxtScanInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TxtScanInfo::HasGoogleAccount => write!(f, "HasGoogleAccount"),
            TxtScanInfo::HasDocusignAccount => write!(f, "HasDocusignAccount"),
            TxtScanInfo::HasAppleAccount => write!(f, "HasAppleAccount"),
            TxtScanInfo::HasFacebookAccount => write!(f, "HasFacebookAccount"),
            TxtScanInfo::HasHubspotAccount => write!(f, "HasHubspotAccount"),
            TxtScanInfo::HasMsDynamics365 => write!(f, "HasMSDynamics365"),
            TxtScanInfo::HasStripeAccount => write!(f, "HasStripeAccount"),
            TxtScanInfo::HasOneTrustSso => write!(f, "HasOneTrustSSO"),
            TxtScanInfo::HasBrevoAccount => write!(f, "HasBrevoAccount"),
            TxtScanInfo::HasGlobalsignAccount => write!(f, "HasGlobalsignAccount"),
            TxtScanInfo::HasGlobalsignSMime => write!(f, "HasGlobalsignSMime"),
            TxtScanInfo::OwnsAtlassianAccounts => write!(f, "OwnsAtlassianAccounts"),
            TxtScanInfo::OwnsZoomAccounts => write!(f, "OwnsZoomAccounts"),
            TxtScanInfo::EmailProtonMail => write!(f, "EmailProtonMail"),
            TxtScanInfo::SPF { parts } => {
                write!(f, "SPF")?;
                for part in parts {
                    write!(f, " {}", part)?;
                }
                Ok(())
            }
        }
    }
}

/// Contains a single parsed TXT line along with the domain it was found on.
#[derive(Debug, Clone)]
pub struct DnsTxtScanResult {
    /// The domain this DNS entry was found on
    pub domain: String,
    /// The record (part) that was matched with this scan result.
    pub rule: String,
    /// The parsed DNS TXT entry
    pub info: TxtScanInfo,
}

/*
some more not yet mapped root domain results:

AkXbiQpYI7uX1sj7+NSmNLAv7t8dX15bc+LseeHs JFX9XIdflE1L8M3US5IfRzqPIUBd9zj1jMEhcl0f c2njJg==
bw=IOlfo6xQJX+xewM7+IiPqOSIPtLXKrWoS2RXCTPMmQZc
fg2t0gov9424p2tdcuo94goe9j
MS=ADD367D1CEC313426372A11C71D893E0B125A F07
MS=CF8A084602474BA62021A3664345E6E1EEB8233E
MS=E4A68B9AB2BB9670BCE15412F62916164C0B20BB
MS=ms15401227
MS=ms71454350
OSSRH-87525
proxy-ssl.webflow.com
t7sebee51jrj7vm932k531hipa
webexdomainverification.8YX6G=6e6922db-e3e6-4a36-904e-a805c28087fa
*/

/// Recursive DNS TXT scan
pub async fn start_dns_txt_scan(
    settings: DnsTxtScanSettings,
    tx: Sender<DnsTxtScanResult>,
) -> Result<(), DnsResolutionError> {
    info!("Started DNS TXT scanning");

    let resolver = TokioAsyncResolver::tokio_from_system_conf()
        .map_err(DnsResolutionError::CreateSystemResolver)?;

    let mut tasks = JoinSet::new();

    for domain in settings.domains {
        scan(&mut tasks, &resolver, &tx, domain);
    }

    while tasks.join_next().await.is_some() {}

    info!("Finished DNS resolution");

    Ok(())
}

fn scan(
    tasks: &mut JoinSet<()>,
    resolver: &ResolverT,
    tx: &Sender<DnsTxtScanResult>,
    domain: String,
) {
    tasks.spawn(domain_impl(resolver.clone(), tx.clone(), domain));
}

async fn process_txt_record(tx: &Sender<DnsTxtScanResult>, domain: &str, record: &[u8]) {
    if record.starts_with(b"v=spf1") {
        tx.send(DnsTxtScanResult {
            domain: domain.to_owned(),
            rule: String::from_utf8_lossy(record).to_string(),
            info: TxtScanInfo::SPF {
                parts: parse_spf(&record[6..]),
            },
        })
        .await
        .ok();
    }

    for txt_type in &BASIC_TXT_TYPES_WITH_REGEX {
        let regex = txt_type.matcher_regex().unwrap();
        if regex.is_match(record) {
            tx.send(DnsTxtScanResult {
                domain: domain.to_owned(),
                rule: String::from_utf8_lossy(record).to_string(),
                info: txt_type.clone(),
            })
            .await
            .ok();
        }
    }
}

async fn recurse_txt(tx: &Sender<DnsTxtScanResult>, domain: &str, records: &[Record]) {
    for record in records {
        if let Some(rdata) = record.data() {
            let txt = rdata.as_txt().unwrap(); // only TXT records allowed
            for data in txt.txt_data() {
                process_txt_record(tx, domain, data).await;
            }
        }
    }
}

async fn domain_impl(resolver: ResolverT, tx: Sender<DnsTxtScanResult>, domain: String) {
    if let Ok(res) = resolve(resolver.txt_lookup(&domain)).await {
        recurse_txt(&tx, &domain, res.as_lookup().records()).await;
    }

    debug!("Finished dns resolution for {}", domain);
}
