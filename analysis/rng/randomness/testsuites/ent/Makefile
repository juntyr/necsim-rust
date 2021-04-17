.SUFFIXES:
#
.SUFFIXES: .cpp .o .c .h


.PHONY: clean cleandist


CFLAGS = -fPIC -std=c99 -O3 -march=native -Wall -Wextra -Wshadow -Wno-implicit-function-declaration


all: ent

random.zip:
	(curl -OL http://www.fourmilab.ch/random/random.zip)

ent: random.zip
	(unzip -qo random.zip -d build && cd build && make && cp ent ..)

clean:
	rm -f ent

cleandist: clean
	rm -r -f rand
