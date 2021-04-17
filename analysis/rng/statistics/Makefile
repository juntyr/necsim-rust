all: turnover.nbconvert.ipynb speciation.nbconvert.ipynb coverage.nbconvert.ipynb

turnover.nbconvert.ipynb: turnover.ipynb
	jupyter nbconvert --to notebook --execute turnover.ipynb

speciation.nbconvert.ipynb: speciation.ipynb
	jupyter nbconvert --to notebook --execute speciation.ipynb

coverage.nbconvert.ipynb: coverage.ipynb
	jupyter nbconvert --to notebook --execute coverage.ipynb
