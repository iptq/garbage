garbage
=======

[![crates.io](https://img.shields.io/crates/v/garbage.svg)](https://crates.io/crates/garbage)
[![dependency status](https://deps.rs/repo/github/iptq/garbage/status.svg)](https://deps.rs/repo/github/iptq/garbage)

Rust ver of trash-cli.

* **Windows/Recycle Bin not supported**

Installation
------------

```
cargo install garbage
```

Usage
-----

Run `garbage --help` to understand how it's used!

```
$ garbage put file1 file2 ...

$ garbage restore
[..interactive]

$ garbage list

$ garbage empty [days]
```

If you use a bash-ish shell, feel free to add this to your shell's rc file:

```sh
alias rm='garbage put' # Make sure garbage is in your path
```

Features
--------

- [x] Put
- [x] List
- [x] Restore
- [ ] Tests...

Spec Compliance
---------------

- [x] Picking a Trash Directory
- [x] Emptying
- [ ] Directory size cache

About
-----

Author: Michael Zhang

License: MIT
