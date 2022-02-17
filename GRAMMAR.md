<datetime> ::= NOW
<datetime> ::= <time>
<datetime> ::= <date> <time>
<datetime> ::= <date>, <time>
<datetime> ::= <duration> AFTER <datetime>
<datetime> ::= A <unit> AFTER <datetime>
<datetime> ::= THE <unit> AFTER <datetime>
<datetime> ::= A <unit> BEFORE <datetime>
<datetime> ::= THE <unit> BEFORE <datetime>
<datetime> ::= <duration> FROM <datetime>
<datetime> ::= RANDOM BETWEEN <datetime> AND <datetime>

<date> ::= TODAY
<date> ::= TOMORROW
<date> ::= YESTERDAY
<date> ::= <num>/<num>/<num>
<date> ::= <num>-<num>-<num>
<date> ::= <month> <num> <num>
<date> ::= <month> <num>, <num>
<date> ::= <relative_specifier> <unit>
<date> ::= <realtive_specifier> <weekday>
<date> ::= <weekday>

<relative_specifier> ::= THIS
                         NEXT

<weekday> ::= MONDAY
              TUESDAY
              WEDNESDAY
              THURSDAY
              FRIDAY
              SATURDAY
              SUNDAY

<month> ::= January
            February
            March
            April
            May
            June
            July
            August
            September
            October
            November
            December

<time> ::= <num>:<num>
<time> ::= <num>:<num> AM
<time> ::= <num>:<num> PM
<time> ::=

<duration> ::= <num> <unit>

<unit> ::= DAY
           WEEK
           HOUR
           MINUTE
           MONTH
           YEAR

<num> ::=
