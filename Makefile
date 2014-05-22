
RUSTC ?= $(shell which rustc)
RUSTDOC ?= $(shell which rustdoc)

# deps
RUST_HTTP    ?= $(shell readlink -f deps/rust-http)
RUST_OPENSSL ?= $(shell readlink -f deps/rust-openssl)
RUST_PPAPI   ?= $(shell readlink -f deps/rust-ppapi)

ifeq ($(USE_DEBUG),0)
RUSTFLAGS += -O --cfg ndebug
CFLAGS += -Oz
CXXFLAGS += -Oz
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

build/ppapi.stamp: $(RUST_PPAPI)/lib.rs Makefile deps/http.stamp
	$(RUSTDOC) $(RUST_PPAPI)/lib.rs -L $(RUST_HTTP)/build -o .
	touch build/ppapi.stamp


# deps

$(RUST_HTTP)/Makefile: $(RUST_HTTP)/configure $(RUST_HTTP)/Makefile.in Makefile
	cd $(RUST_HTTP); \
	./configure

deps/http.stamp: 	$(RUST_HTTP)/Makefile deps/openssl.stamp \
		$(call rwildcard,$(RUST_HTTP),*rs) \
		$(RUSTC)
	make -C $(RUST_HTTP) clean
	RUSTC="$(RUSTC)" RUSTFLAGS="$(RUSTFLAGS)" make -C $(RUST_HTTP)
	touch $@

$(RUST_OPENSSL)/Makefile: $(RUST_OPENSSL)/configure $(RUST_OPENSSL)/Makefile.in Makefile
	cd $(RUST_OPENSSL); \
	./configure

deps/openssl.stamp:	$(RUST_OPENSSL)/Makefile \
		$(call rwildcard,$(RUST_OPENSSL),*rs) \
		$(RUSTC)
	RUSTC="$(RUSTC)" RUSTFLAGS="$(filter-out -O,$(RUSTFLAGS))" make -C $(RUST_OPENSSL)
	touch $@
