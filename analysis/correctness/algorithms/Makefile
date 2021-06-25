.PHONY: clean


all: monolithic.nbconvert.ipynb independent.nbconvert.ipynb cuda.nbconvert.ipynb


monolithic.nbconvert.ipynb: monolithic.ipynb
	jupyter nbconvert --to notebook --execute monolithic.ipynb

independent.nbconvert.ipynb: independent.ipynb
	jupyter nbconvert --to notebook --execute independent.ipynb

cuda.nbconvert.ipynb: cuda.ipynb
	jupyter nbconvert --to notebook --execute cuda.ipynb


clean:
	rm -f monolithic.nbconvert.ipynb independent.nbconvert.ipynb cuda.nbconvert.ipynb
