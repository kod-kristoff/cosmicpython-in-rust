CREATE TABLE IF NOT EXISTS batches
(
    id                    INTEGER PRIMARY KEY NOT NULL,
    reference             STRING(255)         NOT NULL,
    sku                   STRING(255)         NOT NULL,
    _purchased_quantity   INTEGER             NOT NULL,
    eta                   DATETIME
);
