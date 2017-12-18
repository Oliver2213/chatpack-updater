// this file contains the version struct

use chrono::{Local, DateTime, Datelike};

#[derive(Debug)]
pub struct Version {
  year: i16,
  month: i8,
  day: i8,
  patch: i8
}

// implement methods and related functions for the Version struct
impl Version {
  pub fn from_string(version_string: &str) -> Version {
    // this method creates and returns a new version struct;
    // it accepts a string, an example of which is "2017.12.10.1"
    let elements: Vec<&str> = version_string.trim().split('.').collect();
    let year: i16 = elements[0].trim()
      .parse::<i16>()
      .expect("Error converting the year field of version to an integer");
    let month: i8 = elements[1].trim().parse::<i8>()
      .expect("Error converting the month field of version to an integer");
    let day: i8 = elements[2].trim().parse::<i8>()
      .expect("Error converting the day field of version to an integer");
    let patch:i8 = elements[3].trim().parse()
      .expect("Error converting the patch field of version into an integer");
    let v: Version = Version {year, month, day, patch};
    return v;
  }

  pub fn new () -> Version {
    // This function returns a version for the current time, with patch number set to 1
    let now: DateTime<Local> = Local::now();
    let year = now.year() as i16;
    let month = now.month() as i8;
    let day = now.day() as i8;
    let patch: i8 = 1;
    let v = Version {year, month, day, patch};
    return v;
  }

  pub fn update(&mut self) {
    // this method operates on an existing version instance (taking a mutable reference so it can change fields) and updates it;
    // since the version format is year.month.day.patch, this method either sets the first 3 (if `self` represents a different year / month / day)  or increments the patch number

    // get the current time
    let now: DateTime<Local> = Local::now();
    // save the year, month, and day values as i16, i8, and i8, respectively (because that's what `Version` expects)
    let current_year = now.year() as i16;
    let current_month = now.month() as i8;
    let current_day = now.day() as i8;
    // keep track of whether the date fields are updated
    let mut date_updated = false;
    if self.year != current_year {
      self.year = current_year;
      date_updated = true;
    }
    if self.month != current_month {
      self.month = current_month;
      date_updated = true;
    }
    if self.day != current_day {
      self.day = current_day;
      date_updated = true;
    }
    // if the date fields weren't updated, increment the patch number (to indicate that this version is n that day, where n = patch)
    if date_updated == false {
      self.patch = self.patch + 1;
    } else {
      self.patch = 1; // set the patch number to 1 for the new date version
    }
  }
  pub fn to_string(&self) -> String {
    return format!("{}.{}.{}.{}", self.year, self.month, self.day, self.patch);
  }
}