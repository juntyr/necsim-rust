.PHONY: clean

all: dieharder.nbconvert.ipynb ent.nbconvert.ipynb practrand.nbconvert.ipynb testu01.nbconvert.ipynb

dieharder.nbconvert.ipynb: dieharder.ipynb
	jupyter nbconvert --to notebook --execute dieharder.ipynb

ent.nbconvert.ipynb: ent ent.ipynb
	jupyter nbconvert --to notebook --execute ent.ipynb

practrand.nbconvert.ipynb: practrand practrand.ipynb
	jupyter nbconvert --to notebook --execute practrand.ipynb

testu01.nbconvert.ipynb: testu01 testu01.ipynb
	jupyter nbconvert --to notebook --execute testu01.ipynb


ent:
	cd testsuites/ent && make && cp ent ../..

practrand:
	cd testsuites/practrand && make && cp practrand ../..

testu01:
	cd testsuites/testu01 && make && cp testu01 ../..


clean:
	rm -f dieharder.nbconvert.ipynb ent.nbconvert.ipynb practrand.nbconvert.ipynb testu01.nbconvert.ipynb
	rm -f ent practrand testu01
