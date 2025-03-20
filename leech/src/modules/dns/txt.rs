//! leech module for parsing TXT entries in the DNS results specifically.

use std::fmt::Display;
use std::fmt::Formatter;

use hickory_resolver::name_server::GenericConnector;
use hickory_resolver::name_server::TokioRuntimeProvider;
use hickory_resolver::AsyncResolver;
use hickory_resolver::TokioAsyncResolver;
use kraken_proto::any_attack_response::Response;
use kraken_proto::shared;
use kraken_proto::shared::dns_txt_scan::Info;
use kraken_proto::shared::spf_directive;
use kraken_proto::shared::spf_part;
use kraken_proto::shared::DnsTxtKnownService;
use kraken_proto::shared::DnsTxtKnownServiceList;
use kraken_proto::shared::DnsTxtServiceHint;
use kraken_proto::shared::Net;
use kraken_proto::shared::SpfDirective;
use kraken_proto::shared::SpfExplanationModifier;
use kraken_proto::shared::SpfInfo;
use kraken_proto::shared::SpfMechanismA;
use kraken_proto::shared::SpfMechanismAll;
use kraken_proto::shared::SpfMechanismExists;
use kraken_proto::shared::SpfMechanismInclude;
use kraken_proto::shared::SpfMechanismIp;
use kraken_proto::shared::SpfMechanismMx;
use kraken_proto::shared::SpfMechanismPtr;
use kraken_proto::shared::SpfPart;
use kraken_proto::shared::SpfQualifier;
use kraken_proto::shared::SpfRedirectModifier;
use kraken_proto::shared::SpfUnknownModifier;
use kraken_proto::DnsTxtScanRequest;
use kraken_proto::DnsTxtScanResponse;
use log::debug;
use log::info;
use once_cell::sync::Lazy;
use regex::bytes::Regex;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use tonic::Status;

use crate::modules::dns::resolve;
use crate::modules::StreamedAttack;

type ResolverT = AsyncResolver<GenericConnector<TokioRuntimeProvider>>;

use super::errors::DnsResolutionError;
use super::spf::parse_spf;
use super::spf::SPFMechanism;
use super::spf::SPFPart;
use super::spf::SPFQualifier;

pub struct DnsTxtScan;
#[tonic::async_trait]
impl StreamedAttack for DnsTxtScan {
    type Settings = DnsTxtScanSettings;
    type Output = DnsTxtScanResult;
    type Error = DnsResolutionError;
    async fn execute(
        settings: Self::Settings,
        sender: Sender<Self::Output>,
    ) -> Result<(), Self::Error> {
        start_dns_txt_scan(settings, sender).await
    }

    type Request = DnsTxtScanRequest;
    fn get_attack_uuid(request: &Self::Request) -> &str {
        &request.attack_uuid
    }
    fn decode_settings(request: Self::Request) -> Result<Self::Settings, Status> {
        if request.targets.is_empty() {
            return Err(Status::invalid_argument("nothing to resolve"));
        }

        Ok(DnsTxtScanSettings {
            domains: request.targets,
        })
    }

    type Response = DnsTxtScanResponse;
    fn encode_output(output: Self::Output) -> Self::Response {
        DnsTxtScanResponse {
            record: Some(shared::DnsTxtScan {
                domain: output.domain,
                info: Some(match output.info {
                    TxtScanInfo::SPF { parts } => Info::Spf(SpfInfo {
                        parts: parts
                            .iter()
                            .map(|part| SpfPart {
                                rule: part.encode_spf(),
                                part: Some(match part {
                                    SPFPart::Directive {
                                        qualifier,
                                        mechanism,
                                    } => spf_part::Part::Directive(SpfDirective {
                                        mechanism: Some(match mechanism {
                                            SPFMechanism::All => {
                                                spf_directive::Mechanism::All(SpfMechanismAll {})
                                            }
                                            SPFMechanism::Include { domain } => {
                                                spf_directive::Mechanism::Include(
                                                    SpfMechanismInclude {
                                                        domain: domain.clone(),
                                                    },
                                                )
                                            }
                                            SPFMechanism::A {
                                                domain,
                                                ipv4_cidr,
                                                ipv6_cidr,
                                            } => spf_directive::Mechanism::A(SpfMechanismA {
                                                domain: domain.clone(),
                                                ipv4_cidr: ipv4_cidr.map(|a| a as _),
                                                ipv6_cidr: ipv6_cidr.map(|a| a as _),
                                            }),
                                            SPFMechanism::MX {
                                                domain,
                                                ipv4_cidr,
                                                ipv6_cidr,
                                            } => spf_directive::Mechanism::Mx(SpfMechanismMx {
                                                domain: domain.clone(),
                                                ipv4_cidr: ipv4_cidr.map(|a| a as _),
                                                ipv6_cidr: ipv6_cidr.map(|a| a as _),
                                            }),
                                            SPFMechanism::PTR { domain } => {
                                                spf_directive::Mechanism::Ptr(SpfMechanismPtr {
                                                    domain: domain.clone(),
                                                })
                                            }
                                            SPFMechanism::IP { ipnet } => {
                                                spf_directive::Mechanism::Ip(SpfMechanismIp {
                                                    ip: Some(Net::from(*ipnet)),
                                                })
                                            }
                                            SPFMechanism::Exists { domain } => {
                                                spf_directive::Mechanism::Exists(
                                                    SpfMechanismExists {
                                                        domain: domain.clone(),
                                                    },
                                                )
                                            }
                                        }),
                                        qualifier: match qualifier {
                                            SPFQualifier::Pass => SpfQualifier::Pass as _,
                                            SPFQualifier::Fail => SpfQualifier::Fail as _,
                                            SPFQualifier::SoftFail => SpfQualifier::SoftFail as _,
                                            SPFQualifier::Neutral => SpfQualifier::Neutral as _,
                                        },
                                    }),
                                    SPFPart::RedirectModifier { domain } => {
                                        spf_part::Part::Redirect(SpfRedirectModifier {
                                            domain: domain.clone(),
                                        })
                                    }
                                    SPFPart::ExplanationModifier { domain } => {
                                        spf_part::Part::Explanation(SpfExplanationModifier {
                                            domain: domain.clone(),
                                        })
                                    }
                                    SPFPart::UnknownModifier { name, value } => {
                                        spf_part::Part::UnknownModifier(SpfUnknownModifier {
                                            name: name.clone(),
                                            value: value.clone(),
                                        })
                                    }
                                }),
                            })
                            .collect(),
                    }),
                    TxtScanInfo::ServiceHints { hints } => {
                        Info::WellKnown(DnsTxtKnownServiceList {
                            hints: hints
                                .into_iter()
                                .map(|hint| DnsTxtKnownService {
                                    rule: hint.0,
                                    service: match hint.1 {
                                        TxtServiceHint::HasGoogleAccount => {
                                            DnsTxtServiceHint::HasGoogleAccount as _
                                        }
                                        TxtServiceHint::HasDocusignAccount => {
                                            DnsTxtServiceHint::HasDocusignAccount as _
                                        }
                                        TxtServiceHint::HasAppleAccount => {
                                            DnsTxtServiceHint::HasAppleAccount as _
                                        }
                                        TxtServiceHint::HasFacebookAccount => {
                                            DnsTxtServiceHint::HasFacebookAccount as _
                                        }
                                        TxtServiceHint::HasHubspotAccount => {
                                            DnsTxtServiceHint::HasHubspotAccount as _
                                        }
                                        TxtServiceHint::HasMsDynamics365 => {
                                            DnsTxtServiceHint::HasMsDynamics365 as _
                                        }
                                        TxtServiceHint::HasStripeAccount => {
                                            DnsTxtServiceHint::HasStripeAccount as _
                                        }
                                        TxtServiceHint::HasOneTrustSso => {
                                            DnsTxtServiceHint::HasOneTrustSso as _
                                        }
                                        TxtServiceHint::HasBrevoAccount => {
                                            DnsTxtServiceHint::HasBrevoAccount as _
                                        }
                                        TxtServiceHint::HasGlobalsignAccount => {
                                            DnsTxtServiceHint::HasGlobalsignAccount as _
                                        }
                                        TxtServiceHint::HasGlobalsignSMime => {
                                            DnsTxtServiceHint::HasGlobalsignSMime as _
                                        }
                                        TxtServiceHint::OwnsAtlassianAccounts => {
                                            DnsTxtServiceHint::OwnsAtlassianAccounts as _
                                        }
                                        TxtServiceHint::OwnsZoomAccounts => {
                                            DnsTxtServiceHint::OwnsZoomAccounts as _
                                        }
                                        TxtServiceHint::EmailProtonMail => {
                                            DnsTxtServiceHint::EmailProtonMail as _
                                        }
                                    },
                                })
                                .collect(),
                        })
                    }
                }),
            }),
        }
    }

    const BACKLOG_WRAPPER: fn(Self::Response) -> Response = Response::DnsTxtScan;

    fn print_output(output: &Self::Output) {
        match &output.info {
            TxtScanInfo::SPF { parts } => {
                info!("Found SPF entry for {}:", output.domain);
                for part in parts {
                    info!("  {part}");
                }
            }
            _ => {
                info!("Found txt entry for {}: {}", output.domain, output.info);
            }
        };
    }
}

/// DNS TXT scanning settings
#[derive(Debug)]
pub struct DnsTxtScanSettings {
    /// The domains to start resolving TXT settings in
    pub domains: Vec<String>,
}

/// Represents a single parsed DNS TXT entry.
#[derive(Debug, Clone)]
pub enum TxtScanInfo {
    /// Aggregation of all well-known service hint patterns in all the TXT entries for the domain.
    ServiceHints {
        /// List of detected service hints, as tuple (raw TXT record, known service type)
        hints: Vec<(String, TxtServiceHint)>,
    },
    /// /^v=spf1/ and parsed SPF domains & IPs
    SPF {
        /// A list of all successfully parsed SPF parts (unparsable parts simply skipped)
        parts: Vec<SPFPart>,
    },
}

/// A simple service hint with no complex information other than it existing. Indicates possible ownership of third
/// party service accounts or possible control over external services.
#[derive(Debug, Clone, Copy)]
pub enum TxtServiceHint {
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
}

impl Display for TxtServiceHint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TxtServiceHint::HasGoogleAccount => write!(f, "HasGoogleAccount"),
            TxtServiceHint::HasDocusignAccount => write!(f, "HasDocusignAccount"),
            TxtServiceHint::HasAppleAccount => write!(f, "HasAppleAccount"),
            TxtServiceHint::HasFacebookAccount => write!(f, "HasFacebookAccount"),
            TxtServiceHint::HasHubspotAccount => write!(f, "HasHubspotAccount"),
            TxtServiceHint::HasMsDynamics365 => write!(f, "HasMSDynamics365"),
            TxtServiceHint::HasStripeAccount => write!(f, "HasStripeAccount"),
            TxtServiceHint::HasOneTrustSso => write!(f, "HasOneTrustSSO"),
            TxtServiceHint::HasBrevoAccount => write!(f, "HasBrevoAccount"),
            TxtServiceHint::HasGlobalsignAccount => write!(f, "HasGlobalsignAccount"),
            TxtServiceHint::HasGlobalsignSMime => write!(f, "HasGlobalsignSMime"),
            TxtServiceHint::OwnsAtlassianAccounts => write!(f, "OwnsAtlassianAccounts"),
            TxtServiceHint::OwnsZoomAccounts => write!(f, "OwnsZoomAccounts"),
            TxtServiceHint::EmailProtonMail => write!(f, "EmailProtonMail"),
        }
    }
}

static BASIC_TXT_TYPES_WITH_REGEX: [TxtServiceHint; 14] = [
    TxtServiceHint::HasGoogleAccount,
    TxtServiceHint::HasGlobalsignAccount,
    TxtServiceHint::HasGlobalsignSMime,
    TxtServiceHint::HasDocusignAccount,
    TxtServiceHint::HasAppleAccount,
    TxtServiceHint::HasFacebookAccount,
    TxtServiceHint::HasHubspotAccount,
    TxtServiceHint::HasMsDynamics365,
    TxtServiceHint::HasStripeAccount,
    TxtServiceHint::HasOneTrustSso,
    TxtServiceHint::HasBrevoAccount,
    TxtServiceHint::OwnsAtlassianAccounts,
    TxtServiceHint::OwnsZoomAccounts,
    TxtServiceHint::EmailProtonMail,
];

impl TxtServiceHint {
    fn matcher_regex(&self) -> &'static Regex {
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
            TxtServiceHint::HasGoogleAccount => &RE_HAS_GOOGLE_ACCOUNT,
            TxtServiceHint::HasGlobalsignAccount => &RE_HAS_GLOBALSIGN_ACCOUNT,
            TxtServiceHint::HasGlobalsignSMime => &RE_HAS_GLOBALSIGN_SMIME,
            TxtServiceHint::HasDocusignAccount => &RE_HAS_DOCUSIGN_ACCOUNT,
            TxtServiceHint::HasAppleAccount => &RE_HAS_APPLE_ACCOUNT,
            TxtServiceHint::HasFacebookAccount => &RE_HAS_FACEBOOK_ACCOUNT,
            TxtServiceHint::HasHubspotAccount => &RE_HAS_HUBSPOT_ACCOUNT,
            TxtServiceHint::HasMsDynamics365 => &RE_HAS_MS_DYNAMICS365,
            TxtServiceHint::HasStripeAccount => &RE_HAS_STRIPE_ACCOUNT,
            TxtServiceHint::HasOneTrustSso => &RE_HAS_ONE_TRUST_SSO,
            TxtServiceHint::HasBrevoAccount => &RE_HAS_BREVO_ACCOUNT,
            TxtServiceHint::OwnsAtlassianAccounts => &RE_OWNS_ATLASSIAN_ACCOUNTS,
            TxtServiceHint::OwnsZoomAccounts => &RE_OWNS_ZOOM_ACCOUNTS,
            TxtServiceHint::EmailProtonMail => &RE_EMAIL_PROTON_MAIL,
        }
    }
}

impl Display for TxtScanInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TxtScanInfo::ServiceHints { hints } => {
                write!(f, "ServiceHints")?;
                for part in hints {
                    write!(f, " {}", part.1)?;
                }
                Ok(())
            }
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

fn process_txt_record(record: &[u8]) -> Option<TxtScanInfo> {
    if record.starts_with(b"v=spf1") {
        return Some(TxtScanInfo::SPF {
            parts: parse_spf(&record[6..]),
        });
    }

    for txt_type in &BASIC_TXT_TYPES_WITH_REGEX {
        if txt_type.matcher_regex().is_match(record) {
            // take first match for each TXT entry
            return Some(TxtScanInfo::ServiceHints {
                hints: vec![(String::from_utf8_lossy(record).to_string(), *txt_type)],
            });
        }
    }

    None
}

async fn domain_impl(resolver: ResolverT, tx: Sender<DnsTxtScanResult>, domain: String) {
    if let Ok(res) = resolve(resolver.txt_lookup(&domain)).await {
        let records = res.as_lookup().records();
        let mut services = Vec::new();
        for record in records {
            if let Some(rdata) = record.data() {
                let txt = rdata.as_txt().unwrap(); // only TXT records allowed
                for data in txt.txt_data() {
                    let Some(info) = process_txt_record(data) else {
                        continue;
                    };

                    match info {
                        TxtScanInfo::ServiceHints { hints } => {
                            // aggregate all ServiceHints together into one before sending
                            // (since service hints are all shown and thought of as an exhaustive list in a single results page)
                            services.extend(hints);
                        }
                        TxtScanInfo::SPF { .. } => {
                            tx.send(DnsTxtScanResult {
                                domain: domain.clone(),
                                info,
                            })
                            .await
                            .ok();
                        }
                    }
                }
            }
        }

        if !services.is_empty() {
            tx.send(DnsTxtScanResult {
                domain: domain.to_owned(),
                info: TxtScanInfo::ServiceHints { hints: services },
            })
            .await
            .ok();
        }
    }

    debug!("Finished dns resolution for {}", domain);
}
