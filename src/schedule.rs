use chrono::{NaiveDate, NaiveTime};

use crate::bumpix_client::ScheduleResponse;

pub struct Schedule {
    pub date: NaiveDate,
    pub working_hours: Slot,
    pub taken_times: Vec<Slot>,
}

pub struct Slot {
    start: NaiveTime,
    end: NaiveTime,
}

impl Slot {
    pub fn from_midnight_array(array: &[u16; 2]) -> Self {
        Slot {
            start: NaiveTime::from_num_seconds_from_midnight_opt(array[0] as u32 * 60, 0).unwrap(),
            end: NaiveTime::from_num_seconds_from_midnight_opt(array[1] as u32 * 60, 0).unwrap(),
        }
    }
    pub fn in_slot(&self, time: &NaiveTime) -> bool {
        &self.start <= time && time <= &self.end
    }
}

impl Schedule {
    pub fn from_response(response: ScheduleResponse) -> Self {
        let working_hours = Slot::from_midnight_array(&response.time.values().next().unwrap().w);
        let mut taken_times = Vec::new();
        for slot in response.events.values().next().unwrap() {
            // if slot[0] != 540 {
            taken_times.push(Slot::from_midnight_array(slot));
            // } else {
            //     println!("skipping");
            // }
        }
        Schedule {
            date: response.time.keys().next().unwrap().to_naive_date(),
            working_hours,
            taken_times,
        }
    }

    pub fn time_is_free(&self, time: &NaiveTime) -> bool {
        if !self.working_hours.in_slot(time) {
            return false;
        }
        for slot in &self.taken_times {
            if slot.in_slot(time) {
                return false;
            }
        }
        return true;
    }
}
