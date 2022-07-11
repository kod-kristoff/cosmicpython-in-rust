CREATE TABLE IF NOT EXISTS allocations
(
    id             INTEGER PRIMARY KEY NOT NULL,
    orderline_id   INTEGER             NOT NULL,
    batch_id       INTEGER             NOT NULL,
    FOREIGN KEY (orderline_id)
        REFERENCES order_lines (id),
    FOREIGN KEY (batch_id)
        REFERENCES batches (id)
);
