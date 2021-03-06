use crate::Error;
use std::{cmp::Ordering, collections};

#[derive(Debug, Clone)]
pub struct Batch {
    reference: String,
    sku: String,
    eta: Option<chrono::Date<chrono::Utc>>,
    purchased_quantity: u32,
    allocations: collections::HashSet<OrderLine>,
}

impl Batch {
    pub fn new(
        reference: String,
        sku: String,
        qty: u32,
        eta: Option<chrono::Date<chrono::Utc>>,
    ) -> Self {
        let allocations = collections::HashSet::new();
        Self {
            reference,
            sku,
            eta,
            purchased_quantity: qty,
            allocations,
        }
    }

    pub fn with_allocations(
        reference: String,
        sku: String,
        qty: u32,
        eta: Option<chrono::Date<chrono::Utc>>,
        allocations: collections::HashSet<OrderLine>,
    ) -> Self {
        Self {
            reference,
            sku,
            eta,
            purchased_quantity: qty,
            allocations,
        }
    }

    pub fn can_allocate(&self, line: &OrderLine) -> bool {
        self.sku == line.sku && self.available_quantity() >= line.qty
    }

    pub fn allocate(&mut self, line: OrderLine) {
        if self.can_allocate(&line) {
            self.allocations.insert(line);
        }
        //    self.available_quantity -= line.qty;
    }

    pub fn reference(&self) -> &str {
        &self.reference
    }

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn purchased_quantity(&self) -> u32 {
        self.purchased_quantity
    }

    pub fn eta(&self) -> Option<&chrono::Date<chrono::Utc>> {
        self.eta.as_ref()
    }
    pub fn allocations(&self) -> &collections::HashSet<OrderLine> {
        &self.allocations
    }
    pub fn available_quantity(&self) -> u32 {
        self.purchased_quantity - self.allocated_quantity()
    }

    pub fn allocated_quantity(&self) -> u32 {
        self.allocations
            .iter()
            .fold(0u32, |sum, line| sum + line.qty)
    }

    pub fn deallocate(&mut self, line: OrderLine) {
        if self.allocations.contains(&line) {
            self.allocations.remove(&line);
        }
    }
}

impl PartialEq for Batch {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
    }
}

pub fn sort_by_eta(a: &Batch, b: &Batch) -> Ordering {
    if let Some(a_eta) = a.eta() {
        if let Some(b_eta) = b.eta() {
            return a_eta.cmp(b_eta);
        } else {
            return Ordering::Greater;
        }
    } else {
        if b.eta().is_none() {
            return Ordering::Equal;
        } else {
            return Ordering::Less;
        }
    }
}
pub fn allocate<'a>(line: OrderLine, batches: &'a mut [Batch]) -> Result<&'a str, Error> {
    // let mut batches = batches.to_vec();
    batches.sort_by(sort_by_eta);
    for batch in batches {
        if batch.can_allocate(&line) {
            batch.allocate(line);
            return Ok(&batch.reference);
        }
    }
    Err(Error::OutOfStock(line.sku))
}

#[derive(Debug, PartialEq)]
pub enum Response<'a> {
    Ok(&'a str),
    OutOfStock(String),
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
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
        let (mut batch, line) = make_batch_and_line("SMALL-TABLE", 20, 2);

        batch.allocate(line);

        assert_eq!(batch.available_quantity(), 18);
    }

    fn make_batch_and_line(sku: &str, batch_qty: u32, line_qty: u32) -> (Batch, OrderLine) {
        (
            Batch::new(
                "batch-001".to_owned(),
                sku.to_owned(),
                batch_qty,
                Some(chrono::Utc::today()),
            ),
            OrderLine::new("order-123".to_owned(), sku.to_owned(), line_qty),
        )
    }

    fn tomorrow() -> Option<chrono::Date<chrono::Utc>> {
        Some(chrono::Utc::today() + chrono::Duration::days(1))
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
            Some(chrono::Utc::today()),
        );
        let different_sku_line =
            OrderLine::new("order-123".to_owned(), "EXPENSIVE-TOASTER".to_owned(), 10);
        assert!(!batch.can_allocate(&different_sku_line));
    }

    #[test]
    fn can_only_deallocate_allocated_lines() {
        let (mut batch, unallocated_line) = make_batch_and_line("DECORATIVE-TRINKET", 20, 2);
        batch.deallocate(unallocated_line);
        assert_eq!(batch.available_quantity(), 20);
    }

    #[test]
    fn allocation_is_idempotent() {
        let (mut batch, line) = make_batch_and_line("ANGULAR-DESK", 20, 2);
        batch.allocate(line.clone());
        batch.allocate(line);
        assert_eq!(batch.available_quantity(), 18)
    }

    #[test]
    fn prefers_current_stock_batches_to_shipments() {
        let in_stock_batch = Batch::new(
            "in-stock-batch".to_owned(),
            "RETRO-CLOCK".to_owned(),
            100,
            Some(chrono::Utc::today()),
        );
        let shipment_batch = Batch::new(
            "shipment-batch".to_owned(),
            "RETRO-CLOCK".to_owned(),
            100,
            tomorrow(),
        );
        let line = OrderLine::new("oref".to_owned(), "RETRO-CLOCK".to_owned(), 10);

        let mut batches = vec![in_stock_batch, shipment_batch];
        let res = allocate(line, &mut batches);

        assert_eq!(res, Ok("in-stock-batch"));

        assert_eq!(batches[0].available_quantity(), 90);
        assert_eq!(batches[1].available_quantity(), 100);
    }

    #[test]
    fn allocate_returns_outofstock_if_cannot_allocate() {
        let (batch, line) = make_batch_and_line("SMALL-FORK", 10, 10);
        let mut batches = vec![batch];
        allocate(line, &mut batches).expect("");

        let res = allocate(
            OrderLine::new("order2".to_owned(), "SMALL-FORK".to_owned(), 1),
            &mut batches,
        );
        assert_eq!(res, Err(Error::OutOfStock("SMALL-FORK".to_owned())));
    }
}
