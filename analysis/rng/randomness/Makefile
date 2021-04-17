.PHONY: clean

all: ent practrand testu01

ent:
	cd testsuites/ent && make && cp ent ../..

practrand:
	cd testsuites/practrand && make && cp practrand ../..

testu01:
	cd testsuites/testu01 && make && cp testu01 ../..

clean:
	rm -f ent practrand testu01
