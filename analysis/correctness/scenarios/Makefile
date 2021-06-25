.PHONY: clean


all: non-spatial.nbconvert.ipynb spatially-implicit.nbconvert.ipynb almost-infinite.nbconvert.ipynb


non-spatial.nbconvert.ipynb: non-spatial.ipynb
	jupyter nbconvert --to notebook --execute non-spatial.ipynb

spatially-implicit.nbconvert.ipynb: spatially-implicit.ipynb
	jupyter nbconvert --to notebook --execute spatially-implicit.ipynb

almost-infinite.nbconvert.ipynb: almost-infinite.ipynb
	jupyter nbconvert --to notebook --execute almost-infinite.ipynb


clean:
	rm -f non-spatial.nbconvert.ipynb spatially-implicit.nbconvert.ipynb almost-infinite.nbconvert.ipynb
