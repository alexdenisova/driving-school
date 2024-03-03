# driving-school

A utility for signing up for a driving lesson on [Bumpix](https://bumpix.net/). While setting up a crontab, keep in mind that sign-up opens at midnight 2 weeks before the desired date.

## Arguments

* `DS__SIGN_UP_TIME` (needed) - the time to sign up for in 2 weeks.
* `DS__PHONE_NUMBER` (needed) - your phone number for login.
* `DS__PASSWORD` (needed) - your password for login.
* `DS__INSTRUCTOR_ID` (needed) - your instructor's id.
* `DS__DEBUG` (optional, `=false`) - set the level of logging to debug.
