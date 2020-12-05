from multiprocessing import Pool
import subprocess
from itertools import product
import random
import numpy as np
from tqdm import tqdm


def run_simulation(seed, spec, sample, algorithm, radius, sigma):
    with open(
        f"output/stdout_{algorithm}_{seed}_{spec}_{sample}_{radius}_{sigma}.txt", "w"
    ) as output:
        subprocess.run(
            (
                f"../../target/release/rustcoalescence --algorithm {algorithm} --sample {sample} "
                + f"--seed {seed} --speciation {spec} almost-infinite {radius} {sigma}"
            ),
            shell=True,
            stdout=output,
            stderr=subprocess.STDOUT,
        )


rng = np.random.RandomState(
    np.random.MT19937(np.random.SeedSequence(9378470445410382301))
)

rng = np.random.RandomState(
    np.random.MT19937(np.random.SeedSequence(9378470445410382301))
)

algos = ["classical", "gillespie", "skippinggillespie"]
specs = [0.001, 0.0001, 0.00001]
radii = [566]
sigma = [1.0, 4.0, 16.0]
seeds = rng.randint(0, np.iinfo("uint64").max, dtype="uint64", size=100)

jobs = list(product(algos, specs, radii, sigma, seeds))

random.shuffle(jobs)

with tqdm(total=len(jobs)) as progress:

    def update_progress(_):
        progress.update()

    with Pool(40) as pool:
        for algorithm, spec, radius, sigma, seed in jobs:
            pool.apply_async(
                run_simulation,
                (seed, spec, 1.0, algorithm, radius, sigma),
                {},
                update_progress,
                update_progress,
            )

        pool.close()
        pool.join()
