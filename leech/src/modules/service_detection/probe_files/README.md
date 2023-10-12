# Probe files

Probes can be defined via a custom yaml like file format.

**Beware:** Since the parser is completely custom it is very, very strict to reduce its code complexity.

## Examples

```
service: http
prevalence: often
probes:
  - protocol: TCP
    payload_str: HEAD / HTTP/1.1\r\n\r\n
    regex: HTTP/1.[01] \d\d\d [^\r\n]+\r\n
```

```
service: mariadb
prevalence: often
probes:
  - protocol: TCP
    regex: MariaDB
    sub_regex:
      - ^.\x00\x00\x00\x0a(5\.[-_~.+:\w]+MariaDB-[-_~.+:\w]+)\x00
      - ^E\x00\x00\x00(?s-u:.)(?s-u:.)\x04Host .* is not allowed to connect to this MariaDB server
```

## Format

Every file describes how to detect a single service and MUST begin with:
```
service: <name>
prevalence: <often|average|obscure>
probes:
```

It is then followed by a list of probes using 2 spaces per indentation level.

### Probes' values

- `protocol` MUST be one of `TCP`, `UDP`, `TLS` and defines the transport protocol to use for the connection
- `alpn` MAY be set to request an application protocol during the tls handshake (ignored for non-`TLS`)
- `payload_str` MAY be set to a string to be sent upon establishing a connection (conflicts with `payload_b64`)
- `payload_b64` MAY be set to base64 encoded binary to be sent upon establishing a connection (conflicts with `payload_b64`)
- `regex` MUST be a regex, the servers response is matched against
- `sub_regex` MAY be a list of regexes, which is checked after the initial `regex` matched

### Escaping

Each value has to be separated from its key by `: `
and everything in the line after that (excluding the actual line break) is the value.

**Regexes**' escape sequences are resolved by our the [regex](https://docs.rs/regex) library.

**Strings**' escape sequences are resolve by the rust compiler.
You can find a reference in the [rust docs](https://doc.rust-lang.org/reference/tokens.html#ascii-escapes).
However, when you're in doubt while writing a payload, it is generally recommended to use base64 instead.