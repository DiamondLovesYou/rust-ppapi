ifeq ($(SYSROOT),)
$(error I need the sysroot to your Rust build)
endif

RUSTC ?= $(shell readlink -f $(SYSROOT)/bin/rustc)
NACL_SDK  ?= $(shell readlink -f ~/workspace/tools/nacl-sdk/pepper_canary)
RUST_HTTP ?= $(shell readlink -f ../rust-http/build)
ifeq ($(RUST_HTTP),)
$(error I need rust-http! https://github.com/chris-morgan/rust-http.git)
endif

USE_DEBUG ?= 0
RUSTFLAGS += -C cross-path=$(NACL_SDK) -C nacl-flavor=pnacl --target=le32-unknown-nacl -L $(RUST_HTTP) --sysroot=$(SYSROOT)
CC  := $(NACL_SDK)/toolchain/linux_pnacl/bin/pnacl-clang
CXX := $(NACL_SDK)/toolchain/linux_pnacl/bin/pnacl-clang++
CFLAGS += -I$(NACL_SDK)/include -I$(NACL_SDK)/include/pnacl -MMD
CXXFLAGS += -I$(NACL_SDK)/include -I$(NACL_SDK)/include/pnacl -MMD

ifeq ($(USE_DEBUG),0)
RUSTFLAGS += -O --cfg ndebug
CFLAGS += -Oz
CXXFLAGS += -Oz
else
RUSTFLAGS += --debuginfo=2 -Z no-opt
CFLAGS += -g
CXXFLAGS += -g
endif

all: build/ppapi.stamp

clean:
	touch Makefile

build/libhelper.a: helper.cpp Makefile
	mkdir -p build/obj
	$(CXX) $(CXXFLAGS) $< -c -o build/obj/helper.o
	$(AR) cr $@ build/obj/helper.o
-include helper.d

build/ppapi.stamp: lib.rs build/libhelper.a Makefile
	$(RUSTC) $(RUSTFLAGS) lib.rs -C link-args="--pnacl-driver-verbose" --out-dir=build -L $(RUST_HTTP)
	touch build/ppapi.stamp
