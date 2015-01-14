Rusted Pepper
==========

Rust idiomatic bindings to the Pepper API. This API is experimental. Expect it
to change somewhat.

[Documentation](http://diamondlovesyou.github.io/rust-ppapi/doc/ppapi/index.html)

## Build

You'll need to build and install [the PNaCl/NaCl Rust fork](https://github.com/DiamondLovesYou/rust) first.
Then run:

```bash
export NACL_SDK_ROOT=path/to/pepper_39
```

Lastly, run:

```bash
cargo build --target le32-unknown-nacl
```

And profit!

*Don't run ```build.sh```.* It is used to update FFI bindings.

## Getting Started

Taken from [pnacl-hello-world](https://github.com/DiamondLovesYou/rust-pnacl-hello-world):

```rust
#![crate_name = "pnacl-hello-world"]
#![crate_type = "bin"]
#![no_main]

extern crate ppapi;

use std::collections::HashMap;

#[no_mangle]
#[cfg(target_os = "nacl")]
// Called when an instance is created.
// This is called from a new task. It is perfectly "safe" to panic!() here, or in
// any callback (though it will result in instance termination).
pub extern fn ppapi_instance_created(_instance: ppapi::Instance,
                                     _args: HashMap<String, String>) {
    println!("Hello, world!");
}

#[no_mangle]
pub extern fn ppapi_instance_destroyed() {
}
```

## [Pepper.js](https://github.com/google/pepper.js)

Unsupported due to rust-ppapi's use of threads.
