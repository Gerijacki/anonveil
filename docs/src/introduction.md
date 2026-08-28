# Introduction

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

This book covers installation, day-to-day usage, configuration, and —
most importantly — the [threat model](threat-model.md): a precise,
honest account of what AnonVeil protects against and what it doesn't.
Read that chapter before relying on this for anything sensitive.

For the project's source, issue tracker, and contribution guide, see
the [GitHub repository](https://github.com/Gerijacki/anonveil).
