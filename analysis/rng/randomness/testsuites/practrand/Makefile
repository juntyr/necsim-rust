.SUFFIXES:
#
.SUFFIXES: .cpp .o .c .h


.PHONY: clean cleandist


CFLAGS = -fPIC -std=c99 -O3 -Wall -Wextra -Wshadow -Wno-implicit-function-declaration


all: practrand

PractRand-pre0.95.zip:
	(curl -OL https://downloads.sourceforge.net/project/pracrand/PractRand-pre0.95.zip)

practrand: PractRand-pre0.95.zip
	(unzip -qo PractRand-pre0.95.zip -d build && cd build && $(CXX) -std=c++14 -c src/*.cpp src/RNGs/*.cpp src/RNGs/other/*.cpp -O3 -Iinclude -pthread && ar rcs libPractRand.a *.o && $(CXX) -std=c++14 -o practrand tools/RNG_test.cpp libPractRand.a -O3 -Iinclude -pthread && cp practrand ..)

clean:
	rm -f practrand

cleandist: clean
	rm -r -f build
