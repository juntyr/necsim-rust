.PHONY: clean


all: non-spatial.nbconvert.ipynb


non-spatial.nbconvert.ipynb: non-spatial.ipynb
	jupyter nbconvert --to notebook --execute non-spatial.ipynb


clean:
	rm -f non-spatial.nbconvert.ipynb
