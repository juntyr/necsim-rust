.PHONY: clean


all: monolithic.nbconvert.ipynb independent.nbconvert.ipynb


monolithic.nbconvert.ipynb: monolithic.ipynb
	jupyter nbconvert --to notebook --execute monolithic.ipynb

independent.nbconvert.ipynb: independent.ipynb
	jupyter nbconvert --to notebook --execute independent.ipynb


clean:
	rm -f monolithic.nbconvert.ipynb independent.nbconvert.ipynb
