CREATE TYPE PAYMENT_RECURRENCE AS ENUM (
    'continuos',
    'retro'
);

CREATE TYPE PAYMENT_TYPE AS ENUM (
    'ibc',
    'native'
);

CREATE TABLE public_good_funding (
    id SERIAL PRIMARY KEY,
    proposal_id INT NOT NULL,
    payment_recurrence PAYMENT_RECURRENCE NOT NULL,
    payment_type PAYMENT_TYPE NOT NULL,
    receipient VARCHAR NOT NULL,
    amount INT NOT NULL,
    CONSTRAINT fk_proposal_id FOREIGN KEY(proposal_id) REFERENCES governance_proposals(id) ON DELETE CASCADE
);

CREATE INDEX index_public_good_funding_receipient ON public_good_funding (receipient);