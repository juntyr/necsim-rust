.PHONY: clean


all: cuda.nbconvert.ipynb


cuda.nbconvert.ipynb: cuda.ipynb
	jupyter nbconvert --to notebook --execute cuda.ipynb


clean:
	rm -f cuda.nbconvert.ipynb
