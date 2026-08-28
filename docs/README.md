# AnonVeil documentation

AnonVeil is a system-wide Tor transparent proxy and kill switch for
Arch and Debian. It redirects every outbound TCP connection and every
DNS query on the host through Tor, and drops anything that doesn't go
through it — activated and deactivated with a single command, on the
Linux system you already have.

```console
$ sudo anonveil start
:: writing torrc fragment and reloading tor...
:: pointing DNS at Tor...
:: loading kill switch...
:: waiting for tor to finish bootstrapping (this can take a moment)...
✔ AnonVeil is active — all traffic is now routed through Tor.
```

## Contents

- [Installation](installation.md)
- [Usage](usage.md)
- [Configuration](configuration.md)
- [Architecture](architecture.md)
- [Threat model](../threat-model.md) — read this before relying on
  AnonVeil for anything sensitive: a precise, honest account of what it
  protects against and what it doesn't.
- [Troubleshooting](troubleshooting.md)
- [FAQ](faq.md)

For the project's source, issue tracker, and contribution guide, see
the [GitHub repository](https://github.com/Gerijacki/anonveil).
