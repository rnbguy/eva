use ::{NewTask, Task};
use ::errors::Result;
use ::scheduling::{NewTimeSegment, TimeSegment};


pub mod sqlite;


pub trait Database {
    fn add_task(&self, task: NewTask) -> Result<Task>;
    fn remove_task(&self, id: u32) -> Result<()>;
    fn find_task(&self, id: u32) -> Result<Task>;
    fn update_task(&self, task: Task) -> Result<()>;
    fn all_tasks(&self) -> Result<Vec<Task>>;

    fn add_time_segment(&self, time_segment: NewTimeSegment) -> Result<()>;
    fn find_time_segment(&self, id: u32) -> Result<TimeSegment>;
    fn set_time_segment(&self, task: ::Task, time_segment: TimeSegment)
                        -> Result<()>;
    fn all_tasks_per_time_segment(&self)
                                  -> Result<Vec<(TimeSegment, Vec<Task>)>>;
}
