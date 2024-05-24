# Changelog

## WIP

## v0.3.3

### Leech fixes

- Added second probe for the http service which tries a GET if the initial HEAD failed.

## v0.3.2

### Leech fixes

- Ports with random behaviour won't be reported as service

## v0.3.1

### Leech fixes

- TLS services are now detected correctly

## v0.3.0

### Improvements

- Improved tcp service detection's handling of TLS:
  Services protected by SNI will be detected as TLS and aggregated as "unknown"

### SDK Improvements

- Made `add_host` return a single `Uuid`
- Added `add_hosts` which takes a CIDR and returns many `Uuid`s

### Frontend Improvements

- Added a copy uuid button to the edit finding definition view

### Frontend Fixes

- Fixed workspace invites not showing up reliably

### Backend fixes

- Fixed saving notes for finding affected on http-services

### Leech fixes

- Fixed order of probe execution
- Added 1s timeout between each probe to mitigate anti-port-scanning techniques

## v0.2.3

### Backend Fixes

- Fixed http services' relations endpoint

### Frontend Fixes

- Removed scrollbars which might appear when pressing a button

## v0.2.2

### Backend Fixes

- Fixed export of workspaces which contain an http service which is affected by
  at least one finding

### Frontend Fixes

- Fixed hover on an http service's severity

### SDK Fixes

- Fixed sdk method for retrieving domain relations

## v0.2.1

### Backend Fixes

- Updated dependencies to fix vulnerability in rustls:
  https://rustsec.org/advisories/RUSTSEC-2024-0336.html

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
