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
RUSTFLAGS += -C cross-path=$(NACL_SDK) --target=le32-unknown-nacl --sysroot=$(shell readlink -f $(SYSROOT))
TOOLCHAIN ?= $(shell readlink -f $(NACL_SDK)/toolchain/linux_pnacl)
CC  := $(TOOLCHAIN)/bin/pnacl-clang
CXX := $(TOOLCHAIN)/bin/pnacl-clang++
AR  := $(TOOLCHAIN)/bin/pnacl-ar
RANLIB := $(TOOLCHAIN)/bin/pnacl-ranlib
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

all: $(BUILD_DIR)/ppapi.stamp

clean:
	$(MAKE) -C $(LIBRESSL) clean
	touch Makefile

$(BUILD_DIR)/libhelper.a: src/helper.cpp Makefile
	mkdir -p build/obj
	$(CXX) $(CXXFLAGS) $< -c -o build/obj/helper.o
	$(AR) cr $@ build/obj/helper.o
-include helper.d

$(BUILD_DIR)/ppapi.stamp: src/lib.rs $(wildcard src/*.rs) $(BUILD_DIR)/libhelper.a \
	 		  Makefile deps/http.stamp deps/libressl.stamp
	$(RUSTC) $(RUSTFLAGS) $< --out-dir=$(BUILD_DIR) -L $(TOOLCHAIN)/sdk/lib -L $(BUILD_DIR)
	touch $@


# deps

$(RUST_HTTP)/Makefile: $(RUST_HTTP)/configure $(RUST_HTTP)/Makefile.in Makefile
	cd $(RUST_HTTP); \
	WITH_OPENSSL="$(RUST_OPENSSL)" ./configure

deps/http.stamp: $(RUST_HTTP)/Makefile              \
		 deps/openssl.stamp                 \
		 $(call rwildcard,$(RUST_HTTP),*rs) \
		 $(RUSTC)
	$(RM) -f $(RUST_HTTP)/target/.libhttp.timestamp
	RUSTC="$(RUSTC)" RUSTFLAGS="$(RUSTFLAGS) -L $(RUST_OPENSSL)/target" $(MAKE) -C $(RUST_HTTP) SYSROOT=$(shell readlink -f $(SYSROOT))
	cp $(RUST_HTTP)/target/libhttp.rlib $(BUILD_DIR)
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
	cp $(RUST_OPENSSL)/target/libopenssl.rlib $(BUILD_DIR)
	touch $@

deps/libressl.stamp: Makefile                          \
		     $(call rwildcard,$(LIBRESSL),*.c *.h) \
		     $(LIBRESSL)/configure             \
		     $(CC) $(CXX) $(AR)
	cd $(LIBRESSL); \
	CC="$(CC)" CXX="$(CXX)" AR="$(AR)" CFLAGS="$(CFLAGS) -DNO_SYSLOG" CXXFLAGS="$(CXXFLAGS)" \
	RANLIB="$(RANLIB)" ./configure --disable-shared --host=le32-unknown-nacl --without-pic
# keep automake from mucking up the build (this is really, really, F-ing annoying):
	echo "#/bin/sh" > $(LIBRESSL)/config.status
	$(MAKE) -C $(LIBRESSL)/ssl    && cp $(LIBRESSL)/ssl/.libs/libssl.a $(BUILD_DIR)
	$(MAKE) -C $(LIBRESSL)/crypto && cp $(LIBRESSL)/crypto/.libs/libcrypto.a $(BUILD_DIR)
	touch $@
