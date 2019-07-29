ALTER TABLE time_segments RENAME TO old_time_segments;
CREATE TABLE time_segments (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  name TEXT NOT NULL,
  start INTEGER NOT NULL,
  period INTEGER NOT NULL,
  hue INTEGER NOT NULL
);
INSERT INTO time_segments (id, name, start, period, hue)
SELECT id, name, start, period, abs(random()) % 360 FROM old_time_segments;
DROP TABLE old_time_segments;
