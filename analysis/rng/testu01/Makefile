.SUFFIXES:
#
.SUFFIXES: .cpp .o .c .h


.PHONY: clean cleandist

CFLAGS = -fPIC -std=c99 -O3 -Wall -Wextra -Wshadow -Wno-implicit-function-declaration

LIBS=build/lib/libmylib.a build/lib/libprobdist.a build/lib/libtestu01.a
LIBSCMD= build/lib/libtestu01.a build/lib/libprobdist.a build/lib/libmylib.a -lm
LIBTARGET=build/lib/libmylib.a


all: testu01

TestU01.zip:
	(curl -OL http://simul.iro.umontreal.ca/testu01/TestU01.zip)

TestU01-1.2.3: TestU01.zip
	(unzip -qo TestU01.zip)

$(LIBTARGET): TestU01-1.2.3
	(cd TestU01-1.2.3 && ./configure --prefix="$$(dirname "$$(pwd)")/build" && make -j 6 && make -j 6 install)

testu01: $(LIBTARGET) src/main.c
	$(CC) $(CFLAGS) -o testu01 src/main.c $(LIBSCMD) $(LIB_NECSIM_RNG) -Ibuild/include/ -Iinclude

clean:
	rm -f testu01

cleandist: clean
	rm -r -f build
	rm -r -f TestU01-1.2.3
