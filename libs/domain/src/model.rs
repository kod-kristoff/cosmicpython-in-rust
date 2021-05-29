pub struct Batch {
    reference: String,
    sku: String,
    eta: chrono::Date<chrono::Utc>,
    available_quantity: u32,
}

impl Batch {
    pub fn new(reference: String, sku: String, qty: u32, eta: chrono::Date<chrono::Utc>) -> Self {
        Self { reference, sku, eta, available_quantity: qty }
    }

    pub fn can_allocate(&self, line: &OrderLine) -> bool {
        self.sku == line.sku &&
        self.available_quantity >= line.qty
    }

    pub fn allocate(&mut self, line: OrderLine) {
       self.available_quantity -= line.qty;
    }
}

pub struct OrderLine {
    orderid: String,
    sku: String,
    qty: u32,
}

impl OrderLine {
    pub fn new(orderid: String, sku: String, qty: u32) -> Self {
        Self { orderid, sku, qty }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocating_to_a_batch_reduces_the_available_quantity() {
        let mut batch = Batch {
            reference: "batch-001".to_owned(),
            sku: "SMALL-TABLE".to_owned(),
            eta: chrono::Utc::today(),
            available_quantity: 20,
        };
        let line = OrderLine {
            orderid: "order-ref".to_owned(),
            sku: "SMALL-TABLE".to_owned(),
            qty: 2,
        };

        batch.allocate(line);

        assert_eq!(batch.available_quantity, 18);
    }

    fn make_batch_and_line(sku: &str, batch_qty: u32, line_qty: u32) -> (Batch, OrderLine) {
        (
            Batch::new(
                "batch-001".to_owned(),
                sku.to_owned(),
                batch_qty,
                chrono::Utc::today()),
            OrderLine::new(
                "order-123".to_owned(),
                sku.to_owned(),
                line_qty),
        )
    }

    #[test]
    fn can_allocate_if_available_greater_than_required() {
        let (large_batch, small_line) = make_batch_and_line("ELEGANT-LAMP", 20, 2);
        assert!(large_batch.can_allocate(&small_line));
    }

    #[test]
    fn cannot_allocate_if_available_smaller_than_required() {
        let (small_batch, large_line) = make_batch_and_line("ELEGANT-LAMP", 2, 20);
        assert!(!small_batch.can_allocate(&large_line));
    }

    #[test]
    fn can_allocate_if_available_equal_to_required() {
        let (batch, line) = make_batch_and_line("ELEGANT-LAMP", 2, 2);
        assert!(batch.can_allocate(&line));
    }

    #[test]
    fn cannot_allocate_if_skus_do_not_match() {
        let batch = Batch::new(
            "batch-001".to_owned(),
            "UNCOMFORTABLE-CHAIR".to_owned(),
            100,
            chrono::Utc::today());
        let different_sku_line = OrderLine::new(
            "order-123".to_owned(),
            "EXPENSIVE-TOASTER".to_owned(),
            10);
        assert!(!batch.can_allocate(&different_sku_line));
    }
}
