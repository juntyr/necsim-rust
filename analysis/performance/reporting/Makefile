.PHONY: clean


all: almost-infinite.nbconvert.ipynb


almost-infinite.nbconvert.ipynb: almost-infinite.ipynb
	jupyter nbconvert --to notebook --execute almost-infinite.ipynb


clean:
	rm -f almost-infinite.nbconvert.ipynb
