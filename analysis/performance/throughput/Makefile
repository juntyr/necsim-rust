.PHONY: clean


all: throughput.nbconvert.ipynb


throughput.nbconvert.ipynb: throughput.ipynb
	jupyter nbconvert --to notebook --execute throughput.ipynb


clean:
	rm -f throughput.nbconvert.ipynb
