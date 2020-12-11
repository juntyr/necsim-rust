from multiprocessing import Pool
import subprocess
from itertools import product
import random
import numpy as np
from tqdm import tqdm
import re
import os

BIODIVERSITY_PATTERN = re.compile(r"biodiversity of (\d+) unique species")


def run_simulation(seed, spec, sample, algorithm, area, deme):
    if os.path.exists(
        f"output_v2/stdout_{algorithm}_{seed}_{spec}_{sample}_{area}_{deme}.txt"
    ):
        with open(
            f"output_v2/stdout_{algorithm}_{seed}_{spec}_{sample}_{area}_{deme}.txt",
            "r",
        ) as output:
            try:
                biodiversity = int(BIODIVERSITY_PATTERN.search(output.read()).group(1))
                return
            except:
                pass

    with open(
        f"output_v2/stdout_{algorithm}_{seed}_{spec}_{sample}_{area}_{deme}.txt", "w"
    ) as output:
        subprocess.run(
            (
                f"../../target/release/rustcoalescence --algorithm {algorithm} --sample {sample} "
                + f"--seed {seed} --speciation {spec} non-spatial {area} {deme}"
            ),
            shell=True,
            stdout=output,
            stderr=subprocess.STDOUT,
        )


rng = np.random.RandomState(
    np.random.MT19937(np.random.SeedSequence(9378470445410382301))
)

algos = ["cuda"]
specs = [0.001]
areas = [1000]
demes = [1000]
seeds = rng.randint(0, np.iinfo("uint64").max, dtype="uint64", size=5000)

jobs = list(product(algos, specs, areas, demes, seeds))

random.shuffle(jobs)

with tqdm(total=len(jobs)) as progress:

    def update_progress(_):
        progress.update()

    with Pool(8) as pool:
        for algorithm, spec, area, deme, seed in jobs:
            pool.apply_async(
                run_simulation,
                (seed, spec, 1.0, algorithm, area, deme),
                {},
                update_progress,
                update_progress,
            )

        pool.close()
        pool.join()
