Rusted Pepper
==========

Rust idiomatic bindings to the Pepper API. This API is experimental. Expect it
to change somewhat.

## Build

Just run ```make```, or ```remake``` if you're kool. You'll need to pass it two
variables, SYSROOT, pointing to the stage2 directory in the host machine's
target triple in your build of [Rust](https://github.com/DiamondLovesYou/rust),
and NACL_SDK, pointing to ```pepper_canary``` within the NaCl SDK
(```pepper_35``` *might* work, but no promises).

For example on my Ubuntu machine it would be:

```
make SYSROOT=~/workspace/build/rust-pnacl-canary/x86_64-unknown-linux-gnu/stage2 NACL_SDK=~/workspace/tools/nacl-sdk/pepper_canary
```

*Don't run ```build.sh```.* It is used to update FFI bindings.

## Getting Started

Taken from [pnacl-hello-world](https://github.com/DiamondLovesYou/rust-pnacl-hello-world):

```rust
#![crate_name = "pnacl-hello-world"]
#![crate_type = "bin"]
#![no_main]

extern crate ppapi;

use std::collections::hashmap::HashMap;

#[no_mangle]
#[cfg(target_os = "nacl")]
// Called when an instance is created.
// This is called from a new task. It is perfectly "safe" to fail!() here, or in
// any callback (though it will result in instance termination).
pub extern fn ppapi_instance_created(_instance: ppapi::Instance,
                                     _args: HashMap<String, String>) {
    println!("Hello, world!");
}

#[no_mangle]
pub extern fn ppapi_instance_destroyed() {
}
```

Compile with: ```rustc --target le32-unknown-nacl -C cross-path=path/to/pepper/sdk main.rs```

## [More Docs](http://diamondlovesyou.github.io/rust-ppapi/docs/ppapi/index.html)

## [Pepper.js](https://github.com/google/pepper.js)

Unsupported due to rust-ppapi's use of threads.
