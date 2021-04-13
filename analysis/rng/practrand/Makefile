
#.SUFFIXES:
#
#.SUFFIXES: .cpp .o .c .h


.PHONY: clean cleandist
#PROCESSOR:=$(shell uname -m)
#ifeq ($(PROCESSOR), aarch64)
#OPTIFLAG =
#else
#OPTIFLAG = -march=native
#endif

#CFLAGS = -fPIC -std=c99 -O3 $(OPTIFLAG) -Wall -Wextra -Wshadow -Wno-implicit-function-declaration

#HEADERS=include/main.h

all: RNG_test

PractRand-pre0.95.zip:
	(curl -OL https://downloads.sourceforge.net/project/pracrand/PractRand-pre0.95.zip)

RNG_test: PractRand-pre0.95.zip
	(unzip -qo PractRand-pre0.95.zip -d build && cd build && $(CXX) -std=c++14 -c src/*.cpp src/RNGs/*.cpp src/RNGs/other/*.cpp -O3 -Iinclude -pthread &&  ar rcs libPractRand.a *.o &&  $(CXX) -std=c++14 -o RNG_test tools/RNG_test.cpp libPractRand.a -O3 -Iinclude -pthread && cp RNG_test ..)

clean:
	rm -f RNG_test

cleandist: clean
	rm -r -f build
