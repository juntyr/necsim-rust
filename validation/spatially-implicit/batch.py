from multiprocessing import Pool
import subprocess
from itertools import product
import random
import numpy as np
from tqdm import tqdm


def run_simulation(seed, meta_spec, sample, algorithm, local_area, local_deme, meta_area, meta_deme, migration):
    with open(
        f"output/stdout_{algorithm}_{seed}_{migration}_{meta_spec}_{sample}_{local_area}_{local_deme}_{meta_area}_{meta_deme}.txt", "w"
    ) as output:
        subprocess.run(
            (
                f"../../target/release/rustcoalescence --algorithm {algorithm} --sample {sample} "
                + f"--seed {seed} --speciation {meta_spec} spatially-implicit --local-area "
                + f"{local_area} --local-deme {local_deme} --meta-area {meta_area} --meta-deme "
                + f"{meta_deme} --migration {migration}"
            ),
            shell=True,
            stdout=output,
            stderr=subprocess.STDOUT,
        )


rng = np.random.RandomState(
    np.random.MT19937(np.random.SeedSequence(9378470445410382301))
)

algos = ["classical"]#, "gillespie", "skippinggillespie"]
meta_specs = [0.001, 0.0001]
local_areas = [1000, 10000, 100000]
local_demes = [10]
meta_areas = [100, 10000, 1000000]
meta_demes = [10]
migrations = [0.001, 0.01, 0.1, 1.0]
seeds = rng.randint(0, np.iinfo("uint64").max, dtype="uint64", size=1000)

jobs = list(product(algos, meta_specs, local_areas, local_demes, meta_areas, meta_demes, migrations, seeds))

random.shuffle(jobs)

with tqdm(total=len(jobs)) as progress:

    def update_progress(_):
        progress.update()

    with Pool(40) as pool:
        for algorithm, meta_spec, local_area, local_deme, meta_area, meta_deme, migration, seed in jobs:
            pool.apply_async(
                run_simulation,
                (seed, meta_spec, 1.0, algorithm, local_area, local_deme, meta_area, meta_deme, migration),
                {},
                update_progress,
                update_progress,
            )

        pool.close()
        pool.join()
