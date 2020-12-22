CREATE TABLE battles (
  id INTEGER PRIMARY KEY NOT NULL,
  hash VARCHAR UNIQUE NOT NULL,
  hill INTEGER NOT NULL,
  warrior_a INTEGER NOT NULL,
  warrior_b INTEGER NOT NULL,
  a_win INTEGER NOT NULL,
  a_loss INTEGER NOT NULL,
  a_tie INTEGER NOT NULL,
  b_win INTEGER NOT NULL,
  b_loss INTEGER NOT NULL,
  b_tie INTEGER NOT NULL
);