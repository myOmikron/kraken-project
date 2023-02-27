//! This module contains the db queries for the crt.sh database

/// Returns a query for the database of crt.sh
///
/// **Parameter**:
/// - `target`: [&str]: Domain to search
/// - `include_expired`: [bool]: If true, expired certificates are included in the results
pub fn get_query(target: &str, include_expired: bool) -> String {
    if include_expired {
        format!(
            r#"WITH ci AS (
    SELECT min(sub.CERTIFICATE_ID) ID,
           min(sub.ISSUER_CA_ID) ISSUER_CA_ID,
           array_agg(DISTINCT sub.NAME_VALUE) NAME_VALUES,
           x509_commonName(sub.CERTIFICATE) COMMON_NAME,
           x509_notBefore(sub.CERTIFICATE) NOT_BEFORE,
           x509_notAfter(sub.CERTIFICATE) NOT_AFTER,
           encode(x509_serialNumber(sub.CERTIFICATE), 'hex') SERIAL_NUMBER
        FROM (SELECT *
                  FROM certificate_and_identities cai
                  WHERE plainto_tsquery('certwatch', '{target}') @@ identities(cai.CERTIFICATE)
                      AND cai.NAME_VALUE ILIKE ('%' || '{target}' || '%')
                  LIMIT 10000
             ) sub
        GROUP BY sub.CERTIFICATE
)
SELECT ci.ISSUER_CA_ID,
        ca.NAME ISSUER_NAME,
        ci.COMMON_NAME,
        array_to_string(ci.NAME_VALUES, chr(10)) NAME_VALUE,
        ci.ID ID,
        le.ENTRY_TIMESTAMP,
        ci.NOT_BEFORE,
        ci.NOT_AFTER,
        ci.SERIAL_NUMBER
    FROM ci
            LEFT JOIN LATERAL (
                SELECT min(ctle.ENTRY_TIMESTAMP) ENTRY_TIMESTAMP
                    FROM ct_log_entry ctle
                    WHERE ctle.CERTIFICATE_ID = ci.ID
            ) le ON TRUE,
         ca
    WHERE ci.ISSUER_CA_ID = ca.ID
    ORDER BY le.ENTRY_TIMESTAMP DESC NULLS LAST;"#
        )
    } else {
        format!("r#WITH ci AS (
    SELECT min(sub.CERTIFICATE_ID) ID,
           min(sub.ISSUER_CA_ID) ISSUER_CA_ID,
           array_agg(DISTINCT sub.NAME_VALUE) NAME_VALUES,
           x509_commonName(sub.CERTIFICATE) COMMON_NAME,
           x509_notBefore(sub.CERTIFICATE) NOT_BEFORE,
           x509_notAfter(sub.CERTIFICATE) NOT_AFTER,
           encode(x509_serialNumber(sub.CERTIFICATE), 'hex') SERIAL_NUMBER
        FROM (SELECT *
                  FROM certificate_and_identities cai
                  WHERE plainto_tsquery('certwatch', '{target}') @@ identities(cai.CERTIFICATE)
                      AND cai.NAME_VALUE ILIKE ('%' || '{target}' || '%')
                      AND coalesce(x509_notAfter(cai.CERTIFICATE), 'infinity'::timestamp) >= date_trunc('year', now() AT TIME ZONE 'UTC')
                      AND x509_notAfter(cai.CERTIFICATE) >= now() AT TIME ZONE 'UTC'
                  LIMIT 10000
             ) sub
        GROUP BY sub.CERTIFICATE
)
SELECT ci.ISSUER_CA_ID,
        ca.NAME ISSUER_NAME,
        ci.COMMON_NAME,
        array_to_string(ci.NAME_VALUES, chr(10)) NAME_VALUE,
        ci.ID ID,
        le.ENTRY_TIMESTAMP,
        ci.NOT_BEFORE,
        ci.NOT_AFTER,
        ci.SERIAL_NUMBER
    FROM ci
            LEFT JOIN LATERAL (
                SELECT min(ctle.ENTRY_TIMESTAMP) ENTRY_TIMESTAMP
                    FROM ct_log_entry ctle
                    WHERE ctle.CERTIFICATE_ID = ci.ID
            ) le ON TRUE,
         ca
    WHERE ci.ISSUER_CA_ID = ca.ID
    ORDER BY le.ENTRY_TIMESTAMP DESC NULLS LAST;#")
    }
}
