CREATE TABLE time_segments (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);


ALTER TABLE tasks
ADD COLUMN time_segment_id INTEGER REFERENCES time_segments(id);


CREATE TABLE time_ranges (
    id INTEGER NOT NULL,
    time_segment_id INTEGER NOT NULL REFERENCES time_segments(id),
    "from" INTEGER NOT NULL,
    "to" INTEGER NOT NULL,
    PRIMARY KEY (id, time_segment_id)
);
