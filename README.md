Rusted Pepper (Make'em Oxidize!)
==========

Rust idiomatic bindings to the Pepper API.

## Build

Just run ```make``` or ```remake``` if you're kool. You'll need to pass it two
variables, SYSROOT, pointing to your build of
[Rust](https://github.com/DiamondLovesYou/rust), and TOOLCHAIN, pointing to
```pepper_canary``` within the NaCl SDK (```pepper_35``` *might* work, but no
promises).

*Don't run ```build.sh```.* It is used to update FFI bindings.

## Getting Started

Taken from [pnacl-hello-world](https://github.com/DiamondLovesYou/rust-pnacl-hello-world):
```rust
#![crate_name = "pnacl-hello-world")]
#![crate_type = "bin"]
#![no_main]

extern crate ppapi;
extern crate collections;

use collections::hashmap::HashMap;

#[no_mangle]
#[cfg(target_os = "nacl")]
// Called when an instance is created. Return a boxed trait for your callbacks.
pub extern fn ppapi_instance_created
    (instance: ppapi::Instance,
     _args: || -> HashMap<String, String>) -> Box<ppapi::InstanceCallback> {
        use ppapi::ppb::ConsoleInterface;
        let console = instance.console();
        console.log(ppapi::ffi::PP_LOGLEVEL_LOG, "Hello, world!");
        box NoOpt as Box<ppapi::InstanceCallback>
    }

struct NoOpt;
impl ppapi::InstanceCallback for NoOpt { }
```

Compile with: ```rustc --target le32-unknown-nacl -C cross-path=path/to/pepper/sdk -C nacl-flavor=pnacl main.rs```

## More Docs

[Here](http://diamondlovesyou.github.io/rust-ppapi/ppapi/index.html)
