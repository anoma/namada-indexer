CREATE TYPE PAYMENT_RECURRENCE AS ENUM (
    'continuous',
    'retro'
);

CREATE TYPE PAYMENT_KIND AS ENUM (
    'ibc',
    'native'
);

CREATE TABLE public_good_funding (
    id SERIAL PRIMARY KEY,
    proposal_id INT NOT NULL,
    payment_recurrence PAYMENT_RECURRENCE NOT NULL,
    payment_kind PAYMENT_KIND NOT NULL,
    receipient VARCHAR NOT NULL,
    amount NUMERIC(78, 0) NOT NULL,
    CONSTRAINT fk_proposal_id FOREIGN KEY(proposal_id) REFERENCES governance_proposals(id) ON DELETE CASCADE
);

CREATE INDEX index_public_good_funding_receipient ON public_good_funding (receipient);
CREATE UNIQUE INDEX index_public_good_funding_receipient_proposal_id ON public_good_funding (receipient, proposal_id);