ALTER TABLE hill_warriors RENAME TO hill_warriors_temp;

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


INSERT INTO hill_warriors (id, hill, warrior, rank, win, loss, tie, score) 
SELECT id, hill, warrior, rank, win, loss, tie, score FROM hill_warriors_temp;

DROP TABLE hill_warriors_temp;
