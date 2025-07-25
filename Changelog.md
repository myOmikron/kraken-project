# Changelog

## WIP

## 0.7.1

### Backend

- Fixed oauth errors

## 0.7.0

### Backend

- (Admin) Endpoint to clone a workspace

## 0.6.1

### Backend

- Endpoint to retrieve screenshots from kraken

## 0.6.0

### Features leech

- any attack startable from kraken can now be started directly in leech's CLI
  pushing the results to kraken
- any attack startable from kraken has a json output when run directly in
  leech's CLI

### Frontend Fixes

- Finding Definitions are now searchable case-insensitive
- Fixed height of findings table in workspaces

### General

- Kraken is now fully supported to be run via docker
- Leech is on the way

## 0.5.1

- Fixed dns in leech

## 0.5.0

### Feature: Finding Factory

- service detection can auto-create findings for certain services:
    - postgres
    - mariadb
    - snmp
    - ssh

### Frontend Fixes

- Fixed infinite loop in adminworkspaces
- Fixed "add as affected"

### Backend Fixes

- Fixed permissions of state dir creation

### Leech Fixes

- Fixed FTP probe for FileZilla

### General

- Updated dependencies

## v0.4.2

### Frontend

- Fixed scrolling in findings

### Rust

- Updated dependencies

## v0.4.1

### Backend fixes

- Fixed details for finding affecteds not being stored correctly

## v0.4

### Frontend

- Added exportable details in finding and finding affected
- Fixed click zone of some danger zones to only apply on buttons
- Fixed click on cancel button on delete finding popup

### Kraken

- Added exportable details in finding and finding affected

### Leech

- Added openvpn probe

## v0.3.5

### Frontend

- Added version to http services

## v0.3.4

### Frontend fixes

- Clicking on a tag in http-services results now in the filter added in http services instead of services

### General

- Updated dependencies

## v0.3.3

### Leech fixes

- Added second probe for the http service which tries a GET if the initial HEAD failed.

## v0.3.2

### Leech fixes

- Ports with random behaviour won't be reported as service

### General

- Updated dependencies

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
