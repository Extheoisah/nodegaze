use crate::api::common::{FilterRequest, NumericOperator, PaginatedData};
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub payments: PaginatedData<Payment>,
    pub outgoing_payments_amount: f64,
    pub incoming_payments_amount: f64,
    pub outgoing_payment_volume: f64,
    pub incoming_payment_volume: f64,
    pub forwarded_payments_amount: f64,
    pub forwarded_payment_volume: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Payment {
    pub id: String,
    pub amount: f64,
    pub status: PaymentStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    #[default]
    Pending,
    Processing,
    Completed,
    Failed,
}

impl FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "pending" => Ok(PaymentStatus::Pending),
            "processing" => Ok(PaymentStatus::Processing),
            "completed" => Ok(PaymentStatus::Completed),
            "failed" => Ok(PaymentStatus::Failed),
            _ => Err(format!("Unknown payment status: {}", s)),
        }
    }
}

pub type PaymentFilter = FilterRequest<PaymentStatus>;
