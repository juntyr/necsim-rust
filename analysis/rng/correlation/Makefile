.PHONY: clean


all: correlation.nbconvert.ipynb


correlation.nbconvert.ipynb: correlation.ipynb
	jupyter nbconvert --to notebook --execute correlation.ipynb


clean:
	rm -f correlation.nbconvert.ipynb
