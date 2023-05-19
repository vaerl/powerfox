DROP TABLE IF EXISTS config;
DROP TABLE IF EXISTS days;

CREATE TABLE config (
    id uuid PRIMARY KEY NOT NULL,
    cost_heating double precision NOT NULL,
    cost_general double precision NOT NULL,
    monthly_budget_heating double precision NOT NULL,
    monthly_budget_general double precision NOT NULL
);

CREATE TABLE days (
    id uuid PRIMARY KEY NOT NULL,
    heating_consumption double precision NOT NULL,
    general_consumption double precision NOT NULL,
    average_temperature double precision NOT NULL,
    date date NOT NULL
);