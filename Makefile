ifneq ($(MAKECMDGOALS),clean)
ifeq ($(SYSROOT),)
$(error I need the sysroot to your Rust build)
endif
endif

SRC_DIR := $(abspath .)

RUSTC ?= $(shell readlink -f $(SYSROOT)/bin/rustc)
NACL_SDK  ?= $(shell readlink -f ~/workspace/tools/nacl-sdk/pepper_canary)

USE_DEBUG ?= 0
RUSTFLAGS += -C cross-path=$(NACL_SDK) -C nacl-flavor=pnacl --target=le32-unknown-nacl -L $(RUST_HTTP) --sysroot=$(shell readlink -f $(SYSROOT))
TOOLCHAIN ?= $(NACL_SDK)/toolchain/linux_pnacl
CC  := $(TOOLCHAIN)/bin/pnacl-clang
CXX := $(TOOLCHAIN)/bin/pnacl-clang++
AR  := $(TOOLCHAIN)/bin/pnacl-ar
CFLAGS += -I$(NACL_SDK)/include -I$(NACL_SDK)/include/pnacl
CXXFLAGS += -I$(NACL_SDK)/include -I$(NACL_SDK)/include/pnacl

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
	./configure

deps/http.stamp: 	$(RUST_HTTP)/Makefile deps/openssl.stamp \
		$(call rwildcard,$(RUST_HTTP),*rs) \
		$(RUSTC)
	cd $(RUST_HTTP); \
	RUSTC="$(RUSTC)" RUSTFLAGS="$(RUSTFLAGS) -L $(RUST_OPENSSL)/target" $(MAKE) SYSROOT=$(shell readlink -f $(SYSROOT))
	touch $@

$(RUST_OPENSSL)/Makefile: $(RUST_OPENSSL)/configure $(RUST_OPENSSL)/Makefile.in Makefile
	cd $(RUST_OPENSSL); \
	./configure

deps/openssl.stamp:	$(RUST_OPENSSL)/Makefile \
		$(call rwildcard,$(RUST_OPENSSL),*rs) \
		$(RUSTC)
	cd $(RUST_OPENSSL); \
	RUSTC="$(RUSTC)" RUSTFLAGS="$(filter-out -O,$(RUSTFLAGS))" $(MAKE) -C $(RUST_OPENSSL)
	touch $@

deps/libressl.stamp: Makefile                          \
		     $(call rwildcard,$(LIBRESSL),*.c) \
		     $(LIBRESSL)/configure             \
		     $(CC) $(CXX) $(AR)
	cd $(LIBRESSL); \
	CC="$(CC)" CXX="$(CXX)" AR="$(AR)" CFLAGS="$(CFLAGS) -DNO_SYSLOG" CXXFLAGS="$(CXXFLAGS)" ./configure --disable-shared --host=le32-unknown-nacl --without-pic
	$(MAKE) -C $(LIBRESSL)/ssl    && cp $(LIBRESSL)/ssl/.libs/libssl.a $(SRC_DIR)/build
	$(MAKE) -C $(LIBRESSL)/crypto && cp $(LIBRESSL)/crypto/.libs/libcrypto.a $(SRC_DIR)/build
	touch $@

