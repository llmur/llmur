--- 001 Setup

--- Functions
--- Generate updated_at column
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = timezone('utc', now());
    RETURN NEW;
END;
$$ language 'plpgsql';


--- Enums
-- Define the budget rates
CREATE TYPE budget_rate AS ENUM ('hourly', 'daily', 'monthly');

-- CREATE TABLE