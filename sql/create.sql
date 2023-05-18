DROP TABLE IF EXISTS config;
DROP TABLE IF EXISTS days;

CREATE TABLE config (
    id serial PRIMARY KEY NOT NULL,
    cost_heating numeric NOT NULL,
    cost_general numeric NOT NULL,
    monthly_budget_heating numeric NOT NULL,
    monthly_budget_general numeric NOT NULL
);

CREATE TABLE days (
    id uuid PRIMARY KEY NOT NULL,
    heating_consumption double precision NOT NULL,
    general_consumption double precision NOT NULL,
    average_temperature double precision NOT NULL,
    date date NOT NULL
);