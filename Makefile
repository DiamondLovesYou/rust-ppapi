ifneq ($(MAKECMDGOALS),clean)
ifeq ($(SYSROOT),)
$(error I need the sysroot to your Rust build)
endif
endif

SRC_DIR := $(abspath .)
BUILD_DIR ?= $(abspath build)

RUSTC ?= $(shell readlink -f $(SYSROOT)/bin/rustc)
NACL_SDK  ?= $(shell readlink -f ~/workspace/tools/nacl-sdk/pepper_canary)

USE_DEBUG ?= 0
RUSTFLAGS += -C cross-path=$(NACL_SDK) -C nacl-flavor=pnacl --target=le32-unknown-nacl --sysroot=$(shell readlink -f $(SYSROOT))
TOOLCHAIN ?= $(NACL_SDK)/toolchain/linux_pnacl
CC  := $(TOOLCHAIN)/bin/pnacl-clang
CXX := $(TOOLCHAIN)/bin/pnacl-clang++
AR  := $(TOOLCHAIN)/bin/pnacl-ar
CFLAGS += -I$(NACL_SDK)/include -I$(NACL_SDK)/include/pnacl
CXXFLAGS += -I$(NACL_SDK)/include -I$(NACL_SDK)/include/pnacl

export LD_LIBRARY_PATH := $(SYSROOT)/lib:$(LD_LIBRARY_PATH)

# deps
RUST_HTTP    ?= $(shell readlink -f deps/rust-http)
RUST_OPENSSL ?= $(shell readlink -f deps/rust-openssl)
LIBRESSL     ?= $(abspath deps/libressl-2.0.0)

ifeq ($(USE_DEBUG),0)
RUSTFLAGS += -O --cfg ndebug
CFLAGS += -Os
CXXFLAGS += -Os
else
RUSTFLAGS += --debuginfo=2 -Z no-opt
CFLAGS += -g
CXXFLAGS += -g
endif

rwildcard = $(foreach d,$(wildcard $1*),$(call rwildcard,$d/,$2) $(filter $(subst *,%,$2),$d))

.DEFAULT_GOAL := all

all: build/ppapi.stamp

clean:
	$(MAKE) -C $(LIBRESSL) clean
	touch Makefile

build/libhelper.a: helper.cpp Makefile
	mkdir -p build/obj
	$(CXX) $(CXXFLAGS) $< -c -o build/obj/helper.o
	$(AR) cr $@ build/obj/helper.o
-include helper.d

build/ppapi.stamp: lib.rs build/libhelper.a Makefile deps/http.stamp deps/libressl.stamp
	$(RUSTC) $(RUSTFLAGS) lib.rs --out-dir=build -L $(RUST_OPENSSL)/target -L $(RUST_HTTP)/target -L $(RUST_HTTP)/build -L $(TOOLCHAIN)/sdk/lib -L build
	touch build/ppapi.stamp


# deps

$(RUST_HTTP)/Makefile: $(RUST_HTTP)/configure $(RUST_HTTP)/Makefile.in Makefile
	cd $(RUST_HTTP); \
	WITH_OPENSSL="$(RUST_OPENSSL)" ./configure

deps/http.stamp: 	$(RUST_HTTP)/Makefile deps/openssl.stamp \
		$(call rwildcard,$(RUST_HTTP),*rs) \
		$(RUSTC)
	$(RM) -f $(RUST_HTTP)/target/.libhttp.timestamp
	RUSTC="$(RUSTC)" RUSTFLAGS="$(RUSTFLAGS) -L $(RUST_OPENSSL)/target" $(MAKE) -C $(RUST_HTTP) SYSROOT=$(shell readlink -f $(SYSROOT))
	touch $@

$(RUST_OPENSSL)/Makefile: $(RUST_OPENSSL)/configure $(RUST_OPENSSL)/Makefile.in Makefile
	cd $(RUST_OPENSSL); \
	./configure

deps/openssl.stamp:	Makefile                      \
		$(RUST_OPENSSL)/Makefile              \
		$(call rwildcard,$(RUST_OPENSSL),*rs) \
		$(RUSTC)                              \
		deps/libressl.stamp
	cd $(RUST_OPENSSL); \
	RUSTC="$(RUSTC)" RUSTFLAGS="$(filter-out -O,$(RUSTFLAGS)) -L $(BUILD_DIR)" $(MAKE) -C $(RUST_OPENSSL) -B
	touch $@

deps/libressl.stamp: Makefile                          \
		     $(call rwildcard,$(LIBRESSL),*.c *.h) \
		     $(LIBRESSL)/configure             \
		     $(CC) $(CXX) $(AR)
	cd $(LIBRESSL); \
	CC="$(CC)" CXX="$(CXX)" AR="$(AR)" CFLAGS="$(CFLAGS) -DNO_SYSLOG" CXXFLAGS="$(CXXFLAGS)" ./configure --disable-shared --host=le32-unknown-nacl --without-pic
	$(MAKE) -C $(LIBRESSL)/ssl    && cp $(LIBRESSL)/ssl/.libs/libssl.a $(BUILD_DIR)
	$(MAKE) -C $(LIBRESSL)/crypto && cp $(LIBRESSL)/crypto/.libs/libcrypto.a $(BUILD_DIR)
	touch $@

