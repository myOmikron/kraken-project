# Changelog

## v0.2.0

### Features

- HTTP Service as new aggregated model
- introduced Finding Categories as a way to give Findings and Finding Definitions another datapoint
  to identify them. They also are used as grouping for exporting findings via the export API.

### Frontend

- Removed running attacks (temporary)
- List Findings that would be also deleted when deleting a Finding Definition
- Changed default port range of UDP Service detection to 1-6000 instead of 1-65535
- Manual Inserts to Hosts now accepts a CIDR

### SDK

- Implemented Findings, FindingDefinitions, FindingAffected

### Fixes

- Added timeouts to TCP service detection, after initial connection

### Internal changes

- Used ESLint to check thoroughly the code of the frontend
- Made every class component to functional
- Added crate for processing openssl errors
- Made rorm optional for the kraken library, results in faster compile speed of the SDK
- Add trait to convert between DB and API types
- Added custom `CONSOLE` for logging in typescript

## v0.1.0

Initial release of kraken.