# Kraken's filters

Kraken uses a small custom language to filter certain views, most notably the "Data" view in a workspace.

## Grammar

A filter consists of any number of rules where is rule is assigns a value to a key:

!!! example
    `domain: docs.kraken-project.org`

You can also assign a list of values combined using the basic logic operations:

!!! example
    `domains: docs.kraken-project.org, kraken-project.org` (`,` is a logical `OR`)

    `tags: ignore & !ok` (`&` is a logical `AND` and `!` is a logical `NOT`)

!!! note
    Most keys accept both singular and plural forms and there is no semantic difference between them

Some values can (or have to) be expressed in ranges, for example ports.

We use `-` to construct ranges.

The lower and upper bounds are both inclusive and optional.

!!! example
    The following are all semantically equivalent:

    `port: 1-1000`
    `port: -1000`
    `port: !1001-`

### Operator precedence

The filter syntax doesn't support parentheses or similar concepts to modify the order in which logic operations are executed.

`!` is always applied first, `&` second and `,` last.

(The range operator `-` is considered as part of the value and therefore applied before the `!`.)

### String escaping

Strings are not required to be wrapped in `"`.

However, if they are not, the mustn't contain any operator including the `:` used to separate the key from its value.

### Nested filters

Relations such as related ports, domains, hosts, services, etc. are filterable using their respective filter AST definitions. Currently it's only possible to query immediate children and for example not relations of relations.

Nested filters are put in parentheses (`()`). The same syntax rules as with regular filters apply.

!!! example
    In a filter of a host, querying related ports:

    `ports:(port:80) & (port:443)`

    Will match only hosts that have BOTH a port 80 and also a port 443. For other filters they can also be the same if both filters would apply to a given port.

    `ports:(port:80), (port:443)`

    Will match hosts that have EITHER a port 80 or a port 443. This gives the same results as `ports:(port: 80, 443)`, but allows for more specific filtering.

Each nested filter applies to only a single related object at a time. Filter rules that are impossible to satisfy, such as `port: 80 & 443` will never match, since a single port can never have two values at once.

!!! question "Pitfall"

    `ports:(port:80 & 443)` is broken since it tries to match 2 different ports on a single entry line, which never works!

Otherwise the nested filters are computed individually on all the related objects to see if they match or not. Regular boolean logic can be used on the nested filters as a whole to test for existance, not existance and correlation using regular boolean algebra as supported by the syntax. (currently in disjunctive normal form only)

!!! example
    | A  | B  | C  | `(A) & (B), (C) & (B)` |
    |----|----|----|------------------------|
    | ❌ | ❌ | ❌ | ❌                     |
    | ❌ | ❌ | ✅ | ❌                     |
    | ❌ | ✅ | ❌ | ❌                     |
    | ❌ | ✅ | ✅ | ✅                     |
    | ✅ | ❌ | ❌ | ❌                     |
    | ✅ | ❌ | ✅ | ❌                     |
    | ✅ | ✅ | ❌ | ✅                     |
    | ✅ | ✅ | ✅ | ✅                     |

### EBNF (excluding whitespace)
```ebnf
Filter = { Rule };
Rule = Key, ":", Or;
Or = And, { ",", And};
And = Not, { "&", Not };
Not = [ "!" ], ( Value | Range );
Range = [ Value ], "-", [ Value ];
Key = RawString;
Value = EscapedString | RawString;
EscapedString = '"', { ? any character except " ? }, '"';
RawString = { ? any character except the used special characters ? };
```

## Targets and their keys

There are 5 different targets a filter might be applied to:

- `Global` which can be applied everywhere any of the other targets can be applied
- `Domain`
- `Host`
- `Port`
- `Service`
- `HTTP Service`

### Keys

Each target has its own set of allowed keys:

| Target             | Key            | Type                  | Example                                                    |
|--------------------|----------------|-----------------------|------------------------------------------------------------|
| *all targets*      | tag[s]         | string                | `tag: critical`                                            | 
|                    | createdAt      | time range            | `createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"` | 
| **Domain**         | domain[s]      | string                | `domain: docs.kraken-project.org`                          | 
|                    | ip[s]          | ip address or network | `ip: 127.0.0.1`, `ip: 127.0.0.1/24`                        |
|                    | sourceOf       | string                | `sourceOf: docs.kraken-project.org`                        | 
|                    | targetOf       | string                | `targetOf: docs.kraken-project.org`                        | 
| **Host**           | ip[s]          | ip address or network | `ip: 127.0.0.1`, `ip: 127.0.0.1/24`                        |
|                    | os             | os type               | `os: linux`, `os: windows`                                 |
|                    | port[s]        | port number or range  | `port: 80`, ` port: 1-1000`                                |
|                    | service[s]     | string                | `service: http`                                            |
|                    | domain[s]      | string                | `domain: docs.kraken-project.org`                          |
| **Port**           | port[s]        | port number or range  | `port: 80`, `port: 1-1000`                                 |
|                    | ip[s]          | ip address or network | `ip: 127.0.0.1`, `ip: 127.0.0.1/24`                        |
|                    | protocol[s]    | port protocol         | `protocol: tcp`                                            |
|                    | service[s]     | string                | `service: http`                                            |
| **Service**        | service[s]     | string                | `service: http`                                            |
|                    | ip[s]          | ip address or network | `ip: 127.0.0.1`, `ip: 127.0.0.1/24`                        |
|                    | port[s]        | port number or range  | `port: 80`, `port: 1-1000`                                 |
|                    | protocol[s]    | port protocol         | `protocol: tcp`                                            |
|                    | transport[s]   | service transport     | `transport: tls`, `transport: raw`                         |
| **HTTP Service**   | httpService[s] | string                | `httpService: wordpress`                                   |
|                    | ip[s]          | ip address or network | `ip: 127.0.0.1`, `ip: 127.0.0.1/24`                        |
|                    | port[s]        | port number or range  | `port: 80`, `port: 1-1000`                                 |
|                    | domain[s]      | string                | `domain: docs.kraken-project.org`                          |
|                    | basePath[s]    | string                | `basePath: /wp-admin/`                                     |
|                    | tls            | boolean               | `tls: yes`                                                 |
|                    | sni            | boolean               | `sni: yes`                                                 |

### Subkeys

The keys `domain`, `sourceOf`, `targetOf`, `ip`, `port`, `service` and `httpService` each represent a target.

You can use subkeys to query a different property of those targets instead of their main ones.

For example `port.protocol: tcp` on a `Host` would filter for TCP ports instead of a specific port number.

| Target      | Key        | Subkey       | Type                 | Example                                                             |
|-------------|------------|--------------|----------------------|---------------------------------------------------------------------|
| **Domain**  | ip[s]      | tag[s]       | string               | `ip.tag: critical`                                                  | 
|             |            | createdAt    | time range           | `ip.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`       | 
|             |            | os           | os type              | `ip.os: linux`, `ip.os: windows`                                    | 
|             | sourceOf   | tag[s]       | string               | `sourceOf.tag: critical`                                            | 
|             |            | createdAt    | time range           | `sourceOf.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"` | 
|             | targetOf   | tag[s]       | string               | `targetOf.tag: critical`                                            | 
|             |            | createdAt    | time range           | `targetOf.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"` | 
| **Host**    | port[s]    | protocol[s]  | port protocol        | `port.protocol: tcp`                                                |
|             |            | tag[s]       | port protocol        | `port.tag: critical`                                                |
|             |            | createdAt    | time range           | `port.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`     | 
|             | service[s] | port[s]      | port number or range | `service.port: 80`, `service.port: 1-1000`                          |
|             |            | protocol[s]  | port protocol        | `serivce.protocol: tcp`                                             |
|             |            | tag[s]       | port protocol        | `service.tag: critical`                                             |
|             |            | createdAt    | time range           | `service.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`  |
|             |            | transport[s] | service transport    | `service.transport: raw`, `service.transport: tls`                  |
|             | domain[s]  | tag[s]       | string               | `domain.tag: critical`                                              |
|             |            | createdAt    | time range           | `domain.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`   |
| **Port**    | ip[s]      | tag[s]       | string               | `ip.tag: critical`                                                  |
|             |            | createdAt    | time range           | `ip.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`       |
|             |            | os           | os type              | `ip.os:linux`, `ip.os:windows`                                      |
|             | service[s] | tag[s]       | string               | `service.tag: critical`                                             |
|             |            | createdAt    | time range           | `service.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`  |
|             |            | transport[s] | service transport    | `service.transport: raw`, `service.transport: tls`                  |
| **Service** | ip[s]      | tag[s]       | string               | `ip.tag: critical`                                                  |
|             |            | createdAt    | time range           | `ip.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`       |
|             |            | os           | os type              | `ip.os:linux`, `ip.os:windows`                                      |
|             | port[s]    | tag[s]       | string               | `port.tag: critical`                                                |
|             |            | createdAt    | time range           | `port.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`     |
| **HTTP Service** | ip[s] | tag[s]       | string               | `ip.tag: critical`                                                  |
|             |            | createdAt    | time range           | `ip.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`       |
|             |            | os           | os type              | `ip.os:linux`, `ip.os:windows`                                      |
|             | port[s]    | tag[s]       | string               | `port.tag: critical`                                                |
|             |            | createdAt    | time range           | `port.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`     |
|             | domain[s]  | tag[s]       | string               | `domain.tag: critical`                                              | 
|             |            | createdAt    | time range           | `domain.createdAt: "2012-12-12T12:00:00Z"-"2012-12-13T12:00:00Z"`   | 

## Type values

| Type                      | Valid values                                                                                                |
|---------------------------|:------------------------------------------------------------------------------------------------------------|
| **ip address or network** | IPv4 or IPv6 in CIDR notation                                                                               |
| **os type**               | `unknown`, `linux`, `windows`, `apple`, `android`, `freebsd`                                                |
| **port number**           | decimal port number (1-65535) - Usually valid as a range like `1000-2000`                                   |
| **port protocol**         | `tcp`, `udp`, `sctp`, `unknown`                                                                             |
| **service transport**     | `raw`, `tls`                                                                                                |
| **boolean**               | `true` or `yes`, `false` or `no`                                                                            |
| **string**                | any valid string like described above, quoted if it contains whitespace, `,`, `&`, `!`, `:`, `"` or `-`     |
| **time**                  | RFC3339 / ISO 8601 datetime: `"yyyy-mm-ddThh:mm:ssZ"` - Usually valid as a range                            |
|                           | The `T` may be replaced with a space. The timezone must be UTC (`Z`) or a fixed offset (`+0200` / `-01:30`) |
