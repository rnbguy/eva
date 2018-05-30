DROP TABLE time_ranges;


-- Drop `time_segment_id` column from tasks.
BEGIN TRANSACTION;

ALTER TABLE tasks RENAME TO temp_tasks;

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    content TEXT NOT NULL,
    deadline TEXT NOT NULL,
    duration INTEGER NOT NULL,
    importance INTEGER NOT NULL
);

INSERT INTO tasks
SELECT id, content, deadline, duration, importance
FROM temp_tasks;

DROP TABLE temp_tasks;

COMMIT;


DROP TABLE time_segments;
