garbage
=======

[![crates.io](https://img.shields.io/crates/v/garbage.svg)](https://crates.io/crates/garbage)

rust ver of trash-cli, basic functionality is in, code is probably shit (edit: hopefully less shit now)

* **Windows Recycle Bin not supported**

Installation
------------

```
cargo install garbage
```

Usage
-----

```
$ garbage put [-r] file1 file2 ...

$ garbage restore
[..interactive]

$ garbage list

$ garbage empty [days]
```

If you use a bash-ish shell, feel free to add this to your shell's rc file:

```sh
alias rm='$HOME/.cargo/bin/garbage put' # or wherever garbage is
```

Features
--------

- [x] Put
- [ ] List
- [ ] Restore (need to fuck around with DeletionStrategy)
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
