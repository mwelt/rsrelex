DROP TABLE IF EXISTS batch;

CREATE TABLE IF NOT EXISTS batch (
  batch_id INTEGER PRIMARY KEY,
  -- 0:PSO 1:MOPSO
  tpso INTEGER NOT NULL,
  -- 0:F1  
  tfitness INTEGER NOT NULL,
  -- 0:Wikipedia 
  tcorpus INTEGER NOT NULL,
  -- size of corpus
  ncorpus INTEGER NOT NULL,
  -- particles
  nparticles INTEGER NOT NULL,
  -- number of cycles
  ncycles INTEGER NOT NULL,
  -- size of seed set
  nseed INTEGER NOT NULL,
  -- type of seed 0:random 1:fixed
  tseed INTEGER NOT NULL,
  -- number of runs 
  nruns INTEGER NOT NULL
);

INSERT INTO batch (batch_id, tpso, tfitness, tcorpus, ncorpus, nparticles, ncycles, nseed, tseed, nruns)
VALUES
  
  (0, 0, 0, 0, 10000, 100, 50, 11, 0, 50),
  (1, 0, 0, 0, 10000, 100, 50, 5, 0, 100),
  (2, 0, 0, 0, 10000, 100, 50, 5, 1, 50),
  (3, 0, 0, 0, 10000, 100, 50, 8, 0, 50),

  -- single runs with varying corpus sizes
  (10, 0, 0, 0, 500, 100, 50, 5, 1, 1),
  (11, 0, 0, 0, 1000, 100, 50, 5, 1, 1),
  (12, 0, 0, 0, 5000, 100, 50, 5, 1, 1),
  (13, 0, 0, 0, 10000, 100, 50, 5, 1, 1),
  (14, 0, 0, 0, 50000, 100, 50, 5, 1, 1),
  (15, 0, 0, 0, 100000, 100, 50, 5, 1, 1),
  (16, 0, 0, 0, 500000, 100, 50, 5, 1, 1),
  (17, 0, 0, 0, 1000000, 100, 50, 5, 1, 1);

CREATE TABLE IF NOT EXISTS dat (
  -- foreign key to batch
  batch_id INTEGER NOT NULL,
  -- which run \in [0, nruns - 1]
  irun INTEGER NOT NULL,
  -- which cycle in the current run \in [0, ncycle - 1]
  icycle INTEGER NOT NULL,
  -- which particle \in [0, nparticles - 1]
  iparticle INTEGER NOT NULL,

  -- position data
  pos1 REAL NOT NULL,
  pos2 REAL NOT NULL,
  pos3 REAL NOT NULL,
  pos4 REAL NOT NULL,
  pos5 REAL NOT NULL,
  pos6 REAL NOT NULL,

  -- fitness
  fitness REAL NOT NULL,

  -- precision and recall
  "precision" REAL NOT NULL, 
  recall REAL NOT NULL
);
