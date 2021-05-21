.PHONY: clean


all: exponential.nbconvert.ipynb


exponential.nbconvert.ipynb: exponential.ipynb
	jupyter nbconvert --to notebook --execute exponential.ipynb


clean:
	rm -f exponential.nbconvert.ipynb
