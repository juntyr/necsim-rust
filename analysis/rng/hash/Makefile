.PHONY: clean


all: avalanche.nbconvert.ipynb smhasher.nbconvert.ipynb


avalanche.nbconvert.ipynb: avalanche.ipynb
	jupyter nbconvert --to notebook --execute avalanche.ipynb

smhasher.nbconvert.ipynb: smhasher smhasher.ipynb
	jupyter nbconvert --to notebook --execute smhasher.ipynb


build:
	git clone https://github.com/rurban/smhasher.git build && cd build && git submodule init && git submodule update

smhasher: build
	cd build && cmake . && make && cp ./SMHasher ../smhasher


clean:
	rm -f smhasher.nbconvert.ipynb
	rm -f smhasher
