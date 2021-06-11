.PHONY: clean


all: almost-infinite.nbconvert.ipynb nvidia-nsight.nbconvert.ipynb perf.nbconvert.ipynb


almost-infinite.nbconvert.ipynb: almost-infinite.ipynb
	jupyter nbconvert --to notebook --execute almost-infinite.ipynb

nvidia-nsight.nbconvert.ipynb: nvidia-nsight.ipynb
	jupyter nbconvert --to notebook --execute nvidia-nsight.ipynb

perf.nbconvert.ipynb: perf.ipynb
	jupyter nbconvert --to notebook --execute perf.ipynb


clean:
	rm -f almost-infinite.nbconvert.ipynb nvidia-nsight.nbconvert.ipynb perf.nbconvert.ipynb
