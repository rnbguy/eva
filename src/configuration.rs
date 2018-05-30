use std::fmt;

pub use ::scheduling::SchedulingStrategy;


#[derive(Debug)]
pub struct Configuration {
    pub database_path: String,
    pub scheduling_strategy: SchedulingStrategy,
}


impl SchedulingStrategy {
    pub fn as_str(&self) -> &'static str {
        match *self {
            SchedulingStrategy::Importance => "importance",
            SchedulingStrategy::Urgency => "urgency",
        }
    }
}


// impl fmt::Display for SchedulingStrategy {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", match *self {
//             SchedulingStrategy::Importance => "importance",
//             SchedulingStrategy::Urgency => "urgency",
//         })
//     }
// }
