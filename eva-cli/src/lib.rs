use crate::pretty_print::PrettyPrint;
use eva::configuration::Configuration;
use failure::Fail;
use futures::executor::block_on;
use itertools::Itertools;

mod configuration;
mod parse;
mod pretty_print;

use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Debug, Fail)]
enum Error {
    #[fail(display = "{}", _0)]
    Configuration(#[cause] configuration::Error),
    #[fail(display = "{}", _0)]
    Parse(#[cause] parse::Error),
    #[fail(display = "{}", _0)]
    Eva(#[cause] eva::Error),
}

impl From<configuration::Error> for Error {
    fn from(error: configuration::Error) -> Error {
        Error::Configuration(error)
    }
}

impl From<parse::Error> for Error {
    fn from(error: parse::Error) -> Error {
        Error::Parse(error)
    }
}

impl From<eva::Error> for Error {
    fn from(error: eva::Error) -> Error {
        Error::Eva(error)
    }
}

type Result<T> = std::result::Result<T, Error>;

fn set_field(configuration: &Configuration, field: &str, id: u32, value: &str) -> Result<()> {
    let mut task = block_on(eva::get_task(configuration, id))?;
    match field {
        "content" => task.content = value.to_string(),
        "deadline" => task.deadline = parse::deadline(value)?,
        "duration" => task.duration = parse::duration(value)?,
        "importance" => task.importance = parse::importance(value)?,
        _ => unreachable!(),
    };
    Ok(block_on(eva::update_task(configuration, task))?)
}

#[no_mangle]
pub extern "C" fn add(
    content_c: *const c_char,
    deadline_c: *const c_char,
    duration_c: *const c_char,
    importance_c: *const c_char,
) {
    let (content, deadline, duration, importance) = unsafe {
        (
            CStr::from_ptr(content_c).to_str().unwrap(),
            CStr::from_ptr(deadline_c).to_str().unwrap(),
            CStr::from_ptr(duration_c).to_str().unwrap(),
            CStr::from_ptr(importance_c).to_str().unwrap(),
        )
    };
    let new_task = eva::NewTask {
        content: content.to_owned(),
        deadline: parse::deadline(deadline).unwrap(),
        duration: parse::duration(duration).unwrap(),
        importance: parse::importance(importance).unwrap(),
        time_segment_id: 0,
    };
    let configuration = configuration::read().unwrap();
    block_on(eva::add_task(&configuration, new_task)).unwrap();
}

#[no_mangle]
pub extern "C" fn rm(id_c: *const c_char) {
    let id = unsafe { CStr::from_ptr(id_c).to_str().unwrap() };
    let id = parse::id(id).unwrap();
    let configuration = configuration::read().unwrap();
    block_on(eva::delete_task(&configuration, id)).unwrap();
}

#[no_mangle]
pub extern "C" fn set(id_c: *const c_char, field_c: *const c_char, value_c: *const c_char) {
    let (id, field, value) = unsafe {
        (
            CStr::from_ptr(id_c).to_str().unwrap(),
            CStr::from_ptr(field_c).to_str().unwrap(),
            CStr::from_ptr(value_c).to_str().unwrap(),
        )
    };
    let id = parse::id(id).unwrap();
    let configuration = configuration::read().unwrap();
    set_field(&configuration, field, id, value).unwrap();
}

#[no_mangle]
pub extern "C" fn tasks() {
    let configuration = configuration::read().unwrap();
    let tasks = block_on(eva::tasks(&configuration)).unwrap();
    println!("Tasks:");
    for task in &tasks {
        // Indent all lines of task.pretty_print() by two spaces
        println!("  {}", task.pretty_print().split("\n").join("\n  "));
    }
}

#[no_mangle]
pub extern "C" fn schedule(strategy_c: *const c_char) {
    let strategy = unsafe { CStr::from_ptr(strategy_c).to_str().unwrap() };
    let configuration = configuration::read().unwrap();
    let schedule = block_on(eva::schedule(&configuration, &strategy)).unwrap();
    println!("{}", schedule.pretty_print());
}
