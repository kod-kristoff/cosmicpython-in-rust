CREATE TABLE IF NOT EXISTS order_lines
(
    id         INTEGER PRIMARY KEY NOT NULL,
    sku        STRING(255)         NOT NULL,
    qty        INTEGER             NOT NULL,
    orderid    STRING(255)         NOT NULL
);
