//! Functions to insert new attack results and update aggregations

mod bruteforce_subdomains;
mod dns_resolution;
mod host_alive_check;
mod query_certificate_transparency;
mod service_detection;
mod tcp_port_scan;

pub use bruteforce_subdomains::store_bruteforce_subdomains_result;
pub use dns_resolution::store_dns_resolution_result;
pub use host_alive_check::store_host_alive_check_result;
pub use query_certificate_transparency::store_query_certificate_transparency_result;
pub use service_detection::store_service_detection_result;
pub use tcp_port_scan::store_tcp_port_scan_result;
