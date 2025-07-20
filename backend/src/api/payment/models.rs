use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub payments: Vec<Payment>,
    pub outgoing_payments_amount: f64,
    pub incoming_payments_amount: f64,
    pub outgoing_payment_volume: f64,
    pub incoming_payment_volume: f64,
    pub forwarded_payments_amount: f64,
    pub forwarded_payment_volume: f64,
}

#[derive(Debug, Serialize)]
pub struct Payment {
    pub id: String,
    pub amount: f64,
}
