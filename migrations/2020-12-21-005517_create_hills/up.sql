CREATE TABLE hills (
  id INTEGER PRIMARY KEY NOT NULL,
  name VARCHAR UNIQUE NOT NULL,
  key VARCHAR UNIQUE NOT NULL,
  instruction_set INTEGER NOT NULL,
  core_size INTEGER NOT NULL,
  max_cycles INTEGER NOT NULL,
  max_processes INTEGER NOT NULL,
  max_warrior_length INTEGER NOT NULL,
  min_distance INTEGER NOT NULL,
  rounds INTEGER NOT NULL,
  slots INTEGER NOT NULL
);
CREATE TABLE hill_warriors (
  id INTEGER PRIMARY KEY NOT NULL,
  hill INTEGER NOT NULL,
  warrior INTEGER NOT NULL,
  rank INTEGER NOT NULL,
  win REAL NOT NULL,
  loss REAL NOT NULL,
  tie REAL NOT NULL,
  score REAL NOT NULL
);
