# serde_lvm

A library that provides serde-enabled LabVIEW LVM data structures, and can parse them from the LVM data format.

While serialization back to LVM is not currently supported, you can serialize to other serde formats.

## Getting Started

```
extern crate serde_lvm;

use std::fs::File;

fn main() {
  let lvm_file = File::open("my.lvm").unwrap();
  let lvm_data = serde_lvm::from_reader(lvm_file).unwrap();

  // ...
}
```

## Notes

This library is very much in alpha or beta at best, as I have very limited LVM examples to test with. Certain functionality, eg proper handling of escape sequences, is not supported yet.

If you find a bug or other deficiency, please open an issue. Pull Requests are also welcome.
