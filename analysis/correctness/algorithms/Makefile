.PHONY: clean


all: monolithic.nbconvert.ipynb


monolithic.nbconvert.ipynb: monolithic.ipynb
	jupyter nbconvert --to notebook --execute monolithic.ipynb


clean:
	rm -f monolithic.nbconvert.ipynb
