use std::io;

use chrono::prelude::*;
use chrono::Duration;
use diesel;
use diesel::prelude::*;

use ::errors::*;
use ::configuration::Configuation;
use ::scheduling;
use super::Database;

use self::tasks::dsl::tasks as task_table;
use self::time_segments::dsl::time_segments as time_segment_table;
use self::time_ranges::dsl::time_ranges as time_range_table;


#[derive(Debug, Clone, PartialEq, Queryable, Identifiable, Associations)]
#[belongs_to(TimeSegment)]
#[table_name="tasks"]
struct Task {
    id: i32,
    content: String,
    deadline: i32,
    duration: i32,
    importance: i32,

    time_segment_id: Option<i32>,
}

#[derive(Debug, Insertable)]
#[table_name="tasks"]
struct NewTask {
    content: String,
    deadline: i32,
    duration: i32,
    importance: i32,
}

#[derive(Debug, Identifiable, AsChangeset)]
#[table_name="tasks"]
struct UpdatedTask {
    id: i32,
    content: String,
    deadline: i32,
    duration: i32,
    importance: i32,
}

#[derive(Debug, Queryable, Identifiable)]
#[table_name="time_segments"]
struct TimeSegment {
    id: i32,
    name: String,
}

#[derive(Debug, Insertable)]
#[table_name="time_segments"]
struct NewTimeSegment {
    name: String,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(TimeSegment)]
#[table_name="time_ranges"]
struct TimeRange {
    id: i32,
    time_segment_id: i32,
    from: i32,
    to: i32,
}

#[derive(Debug, Insertable)]
#[table_name="time_ranges"]
struct NewTimeRange {
    time_segment_id: i32,
    from: i32,
    to: i32,
}


table! {
    tasks {
        id -> Integer,
        content -> Text,
        deadline -> Integer,
        duration -> Integer,
        importance -> Integer,
        time_segment_id -> Nullable<Integer>,
    }
}

table! {
    time_segments {
        id -> Integer,
        name -> VarChar,
    }
}

table! {
    time_ranges (id, time_segment_id) {
        id -> Integer,
        time_segment_id -> Integer,
        from -> Integer,
        to -> Integer,
    }
}

embed_migrations!();

no_arg_sql_function!(last_insert_rowid, diesel::sql_types::Integer);


impl Database for SqliteConnection {
    fn add_task(&self, task: ::NewTask) -> Result<::Task> {
        diesel::insert_into(task_table)
            .values(&NewTask::from(task))
            .execute(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to add a task".to_owned()))?;
        let id: i32 = diesel::select(last_insert_rowid)
            .get_result(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to fetch the id of the new task".to_owned()))?;
        self.find_task(id as u32)
    }

    fn remove_task(&self, id: u32) -> Result<()> {
        let amount_deleted =
            diesel::delete(task_table.find(id as i32))
            .execute(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to remove a task".to_owned()))?;
        ensure!(amount_deleted == 1,
                ErrorKind::Database(
                    "while trying to remove a task".to_owned()));
        Ok(())
    }

    fn find_task(&self, id: u32) -> Result<::Task> {
        let db_task: Task = task_table.find(id as i32)
            .get_result(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to find a task".to_owned()))?;
        Ok(::Task::from(db_task))
    }

    fn update_task(&self, task: ::Task) -> Result<()> {
        let db_task = UpdatedTask::from(task);
        let amount_updated =
            diesel::update(&db_task)
            .set(&db_task)
            .execute(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to update a task".to_owned()))?;
        ensure!(amount_updated == 1,
                ErrorKind::Database(
                    "while trying to update a task".to_owned()));
        Ok(())
    }

    fn all_tasks(&self) -> Result<Vec<::Task>> {
        let db_tasks = task_table.load::<Task>(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to retrieve tasks".to_owned()))?;
        Ok(db_tasks.into_iter()
           .map(|task| ::Task::from(task))
           .collect())
    }

    fn add_time_segment(&self, time_segment: scheduling::NewTimeSegment
    ) -> Result<()> {
        diesel::insert_into(time_segment_table)
            .values(&NewTimeSegment::from(time_segment))
            .execute(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to add a time segment".to_owned()))?;
        let id: i32 = diesel::select(last_insert_rowid)
            .get_result(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to fetch the id of the new time segment".to_owned()))?;
        diesel::insert_into(time_range_table)
            .values(&Vec::from((
                TimeSegment { id, name: time_segment.name },
                time_segment)))
            .execute(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to save the ranges of a time segment".to_owned()))?;
        self.find_time_segment(id as u32)
    }

    fn find_time_segment(&self, id: u32) -> Result<scheduling::TimeSegment> {
        let db_time_segment: TimeSegment = time_segment_table.find(id as i32)
            .get_result(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to find a time segment".to_owned()))?;
        let db_time_ranges = TimeRange::belonging_to(&db_time_segment)
            .load::<TimeRange>(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to retrieve time ranges".to_owned()))?;
        Ok(::TimeSegment::from((db_time_segment, db_time_ranges)))
    }

    fn set_time_segment(&self,
                        task: ::Task,
                        time_segment: scheduling::TimeSegment
    ) -> Result<()> {
        let amount_updated =
            diesel::update(task_table.find(task.id as i32))
            .set(task_table::columns::time_segment_id.equals(time_segment.id))
            .execute(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to update a task".to_owned()))?;
        ensure!(amount_updated == 1,
                ErrorKind::Database(
                    "while trying to update a task".to_owned()));
        Ok(())
    }

    fn all_tasks_per_time_segment(&self
    ) -> Result<Vec<(scheduling::TimeSegment, Vec<::Task>)>> {
        let db_time_segments = time_segment_table.load::<TimeSegment>(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to retrieve time segments".to_owned()))?;
        let db_time_ranges = TimeRange::belonging_to(&db_time_segments)
            .load::<TimeRange>(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to retrieve time segments".to_owned()))?
            .grouped_by(&db_time_segments);
        let db_tasks = Task::belonging_to(&db_time_segments)
            .load::<Task>(self)
            .chain_err(|| ErrorKind::Database(
                "while trying to retrieve time segments".to_owned()))?
            .grouped_by(&db_time_segments);

        let time_segments = db_time_segments.into_iter()
            .zip(db_time_ranges)
            .map(|(segment, ranges)| scheduling::TimeSegment::from((segment, ranges)));
        let tasks = db_tasks.into_iter()
            .map(|tasks| tasks.into_iter().map(|task| ::Task::from(task)).collect());
        Ok(time_segments.into_iter().zip(tasks).collect())
    }
}

impl From<::NewTask> for NewTask {
    fn from(task: ::NewTask) -> NewTask {
        NewTask {
            content: task.content,
            deadline: task.deadline.timestamp() as i32,
            duration: task.duration.num_seconds() as i32,
            importance: task.importance as i32,
        }
    }
}

impl From<Task> for ::Task {
    fn from(task: Task) -> ::Task {
        ::Task {
            id: task.id as u32,
            content: task.content,
            deadline: timestamp_to_datetime(task.deadline),
            duration: Duration::seconds(i64::from(task.duration)),
            importance: task.importance as u32,
        }
    }
}

impl From<::Task> for UpdatedTask {
    fn from(task: ::Task) -> UpdatedTask {
        UpdatedTask {
            id: task.id as i32,
            content: task.content,
            deadline: task.deadline.timestamp() as i32,
            duration: task.duration.num_seconds() as i32,
            importance: task.importance as i32,
        }
    }
}

impl From<(TimeSegment, Vec<TimeRange>)> for scheduling::TimeSegment {
    fn from((time_segment, time_ranges): (TimeSegment, Vec<TimeRange>))
            -> scheduling::TimeSegment {
        let ranges = time_ranges
            .map(|range| timestamp_to_datetime(range.from)..timestamp_to_datetime(range.to))
            .collect();
        scheduling::TimeSegment {
            id: time_segment.id,
            name: time_segment.name,
            ranges: ranges,
        }
    }
}

fn From<scheduling::NewTimeSegment> for NewTimeSegment {
    fn from(time_segment: scheduling::NewTimeSegment) -> NewTimeSegment {
        NewTimeSegment { name: time_segment.name }
    }
}

// fn From<(TimeSegment, scheduling::NewTimeSegment)> for Vec<NewTimeRange> {
//     fn from((db_segment, segment): (TimeSegment, scheduling::NewTimeSegment))
//             -> Vec<NewTimeRange> {
//         time_segment.ranges
//             .into_iter()
//             .map(|range| NewTimeRange::from((db_segment, range)))
//             .collect()
//     }
// }

// fn From<(TimeSegment, Range<DateTime<Local>>)> for NewTimeRange {
//     fn from((time_segment, range): (TimeSegment, Range<DateTime<Local>>))
//             -> NewTimeRange {
//         NewTimeRange {
//             time_segment_id: time_segment.id,
//             from: range.from.timestamp(),
//             to: range.to.timestamp(),
//         }
//     }
// }

fn timestamp_to_datetime(timestamp: i32) -> DateTime<Local> {
    let naive_datetime = NaiveDateTime::from_timestamp(i64::from(timestamp), 0);
    Local.from_utc_datetime(&naive_datetime)
}


pub fn make_connection(configuration: &Configuration) -> Result<SqliteConnection> {
    make_connection_with(&configuration.database_path)
}

fn make_connection_with(database_url: &str) -> Result<SqliteConnection> {
    let connection = SqliteConnection::establish(database_url)
        .chain_err(|| ErrorKind::Database(format!("while trying to connect to {}", database_url)))?;
    // TODO run instead of run_with_output
    embedded_migrations::run_with_output(&connection, &mut io::stderr())
        .chain_err(|| ErrorKind::Database("while running migrations".to_owned()))?;
    Ok(connection)
}



#[cfg(test)]
mod tests {
    use super::*;

    use chrono::offset::LocalResult;

    #[test]
    fn test_insert_query_and_delete_single_task() {
        let connection = make_connection_with(":memory:").unwrap();

        // Fresh database has no tasks
        assert_eq!(connection.all_tasks().unwrap().len(), 0);

        // Inserting a task and querying for it, returns the same one
        let new_task = test_task();
        connection.add_task(new_task.clone()).unwrap();
        let tasks = connection.all_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].content, new_task.content);
        assert_eq!(tasks[0].deadline.timestamp(), new_task.deadline.timestamp());
        assert_eq!(tasks[0].duration, new_task.duration);
        assert_eq!(tasks[0].importance, new_task.importance);
        let same_task = connection.find_task(tasks[0].id).unwrap();
        assert_eq!(same_task.content, new_task.content);
        assert_eq!(same_task.deadline.timestamp(), new_task.deadline.timestamp());
        assert_eq!(same_task.duration, new_task.duration);
        assert_eq!(same_task.importance, new_task.importance);

        // Removing a task leaves the database empty
        connection.remove_task(tasks[0].id).unwrap();
        assert!(connection.all_tasks().unwrap().is_empty());
    }

    #[test]
    fn test_insert_update_query_single_task() {
        let connection = make_connection_with(":memory:").unwrap();

        let new_task = test_task();
        connection.add_task(new_task).unwrap();

        let mut tasks = connection.all_tasks().unwrap();
        let mut task = tasks.pop().unwrap();
        let deadline = Local.from_utc_datetime(
            &NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap());
        task.content = "stuff".to_string();
        task.deadline = deadline;
        task.duration = Duration::minutes(7);
        task.importance = 100;
        connection.update_task(task.clone()).unwrap();

        let task_from_db = connection.find_task(task.id).unwrap();
        assert_eq!(task, task_from_db);
        assert_eq!(task.content, "stuff");
        assert_eq!(task.deadline, deadline);
        assert_eq!(task.duration, Duration::minutes(7));
        assert_eq!(task.importance, 100);
    }

    #[test]
    fn test_all_tasks_per_time_segment() {
        let connection = make_connection_with(":memory:").unwrap();

        let task1 = test_task();
        let task2 = test_task();
        let task3 = test_task();
        for task in vec![task1, task2, task3] {
            connection.add_task(task).unwrap();
        }

        let time_segment1 = test_time_segment();
        let time_segment2 = test_time_segment();
        for time_segment in vec![time_segment1, time_segment2] {
            connection.add_time_segment(time_segment).unwrap();
        }
        connection.set_time_segment(&mut task1, time_segment1);
        connection.set_time_segment(&mut task2, time_segment1);
        connection.set_time_segment(&mut task3, time_segment2);

        let tasks_per_time_segment = connection.all_tasks_per_time_segment().unwrap();
        assert_eq!(tasks_per_time_segment[0].0, time_segment1);
        assert_eq!(tasks_per_time_segment[0].1[0], task1);
        assert_eq!(tasks_per_time_segment[0].1[1], task2);
        assert_eq!(tasks_per_time_segment[1].0, time_segment2);
        assert_eq!(tasks_per_time_segment[1].1[0], task3);
    }

    fn test_task() -> ::NewTask {
        ::NewTask {
            content: "do me".to_string(),
            deadline: Local::now(),
            duration: Duration::seconds(6),
            importance: 42,
        }
    }

    fn test_time_segment() -> ::scheduling::NewTimeSegment {
        ::scheduling::NewTimeSegment {
            name: "at work",
            ranges: vec![Local.ymd(2017, 5, 1)..Local::now()],
        }
    }
}
