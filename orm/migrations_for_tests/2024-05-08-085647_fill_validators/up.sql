-- Creating a function to generate random strings
CREATE OR REPLACE FUNCTION random_string(length INT)
RETURNS VARCHAR AS $$
BEGIN
  RETURN array_to_string(array(select substring('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789' from (random()*62 + 1)::int for 1) from generate_series(1,length)), '');
END;
$$ LANGUAGE plpgsql;

-- Starting the DO block
DO $$
DECLARE
    i INT := 1;
BEGIN
    -- Loop to insert 500 random validators
    WHILE i <= 500 LOOP
        INSERT INTO validators (namada_address, voting_power, max_commission, commission, email, website, description, discord_handle, avatar, epoch)
        VALUES (
            random_string(64), -- Random Namada address
            floor(random()*10000) + 1, -- Random voting power
            (random()*100)::text || '%', -- Random max commission
            (random()*100)::text || '%', -- Random commission
            random_string(10) || '@example.com', -- Random email address
            'https://example.com', -- Constant website address
            random_string(50), -- Random description
            '@' || random_string(10), -- Random Discord handle
            'https://example.com/avatar.jpg', -- Constant avatar link
            1 -- Constant epoch
        );
        i := i + 1;
    END LOOP;
END $$;
