-- Initializes workout_summary and measurements tables

CREATE TABLE WORKOUT_SUMMARY (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    km_ridden NUMERIC NOT NULL,
    avg_speed NUMERIC NOT NULL,
    avg_watts NUMERIC NOT NULL,
    avg_rpm NUMERIC NOT NULL,
    avg_heartrate NUMERIC NOT NULL
);

CREATE TABLE MEASUREMENTS (
    speed FLOAT NOT NULL,
    watts INTEGER NOT NULL,
    rpm INTEGER NOT NULL,
    resistance INTEGER NOT NULL,
    heartrate INTEGER NOT NULL,
    workout_id INTEGER NOT NULL,
    CONSTRAINT MEASUREMENTS_WORKOUT_SUMMARY_FK FOREIGN KEY (workout_id) REFERENCES WORKOUT_SUMMARY(id)
);
