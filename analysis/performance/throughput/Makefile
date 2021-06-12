.PHONY: clean


all: throughput.nbconvert.ipynb perf.nbconvert.ipynb


throughput.nbconvert.ipynb: throughput.ipynb
	jupyter nbconvert --to notebook --execute throughput.ipynb


perf.nbconvert.ipynb: perf.ipynb
	jupyter nbconvert --to notebook --execute perf.ipynb

clean:
	rm -f throughput.nbconvert.ipynb perf.nbconvert.ipynb
