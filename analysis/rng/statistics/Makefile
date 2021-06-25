.PHONY: clean


all: turnover.nbconvert.ipynb speciation.nbconvert.ipynb non-spatial-coverage.nbconvert.ipynb normal-dispersal.nbconvert.ipynb


turnover.nbconvert.ipynb: turnover.ipynb
	jupyter nbconvert --to notebook --execute turnover.ipynb

speciation.nbconvert.ipynb: speciation.ipynb
	jupyter nbconvert --to notebook --execute speciation.ipynb

non-spatial-coverage.nbconvert.ipynb: non-spatial-coverage.ipynb
	jupyter nbconvert --to notebook --execute non-spatial-coverage.ipynb

normal-dispersal.nbconvert.ipynb: normal-dispersal.ipynb
	jupyter nbconvert --to notebook --execute normal-dispersal.ipynb


clean:
	rm -f turnover.nbconvert.ipynb speciation.nbconvert.ipynb non-spatial-coverage.nbconvert.ipynb normal-dispersal.nbconvert.ipynb
