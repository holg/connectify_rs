// --- File: crates/connectify_common/src/services.rs ---
//! Service abstractions for external services.
//!
//! This module provides trait definitions for external services used by the application.
//! These traits allow for dependency injection and easier testing by decoupling the
//! application logic from specific implementations of external services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for a boxed future that returns a Result
pub type BoxFuture<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>;

/// A wrapper error type that implements std::error::Error for Box<dyn std::error::Error + Send + Sync>
#[derive(Debug)]
pub struct BoxedError(pub Box<dyn StdError + Send + Sync>);

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for BoxedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl From<Box<dyn StdError + Send + Sync>> for BoxedError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        BoxedError(err)
    }
}

/// A trait for calendar service operations.
///
/// This trait defines the operations that can be performed on a calendar service,
/// such as checking availability, booking slots, and managing events.
pub trait CalendarService: Send + Sync {
    /// Error type returned by calendar service operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get busy time intervals within a specified time range.
    #[allow(clippy::type_complexity)]
    fn get_busy_times(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> BoxFuture<'_, Vec<(DateTime<Utc>, DateTime<Utc>)>, Self::Error>;

    /// Create a calendar event.
    fn create_event(
        &self,
        calendar_id: &str,
        event: CalendarEvent,
    ) -> BoxFuture<'_, CalendarEventResult, Self::Error>;

    /// Delete a calendar event.
    fn delete_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        notify_attendees: bool,
    ) -> BoxFuture<'_, (), Self::Error>;

    /// Mark a calendar event as cancelled.
    fn mark_event_cancelled(
        &self,
        calendar_id: &str,
        event_id: &str,
        notify_attendees: bool,
    ) -> BoxFuture<'_, CalendarEventResult, Self::Error>;

    /// Get booked events within a specified time range.
    #[allow(clippy::type_complexity)]
    fn get_booked_events(
        &self,
        calendar_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        include_cancelled: bool,
    ) -> BoxFuture<'_, Vec<BookedEvent>, Self::Error>;
}

/// A trait for payment service operations.
///
/// This trait defines the operations that can be performed on a payment service,
/// such as creating charges, managing subscriptions, and handling refunds.
pub trait PaymentService: Send + Sync {
    /// Error type returned by payment service operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create payment intent.
    fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        description: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> BoxFuture<'_, PaymentIntentResult, Self::Error>;

    /// Confirm payment intent.
    fn confirm_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> BoxFuture<'_, PaymentIntentResult, Self::Error>;

    /// Cancel a payment intent.
    fn cancel_payment_intent(
        &self,
        payment_intent_id: &str,
    ) -> BoxFuture<'_, PaymentIntentResult, Self::Error>;

    /// Create a refund.
    fn create_refund(
        &self,
        payment_intent_id: &str,
        amount: Option<i64>,
        reason: Option<&str>,
    ) -> BoxFuture<'_, RefundResult, Self::Error>;
}

/// A trait for notification service operations.
///
/// This trait defines the operations that can be performed on a notification service,
/// such as sending emails, SMS, or push notifications.
pub trait NotificationService: Send + Sync {
    /// Error type returned by notification service operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send an email notification.
    fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        is_html: bool,
    ) -> BoxFuture<'_, NotificationResult, Self::Error>;

    /// Send an SMS notification.
    fn send_sms(&self, to: &str, body: &str) -> BoxFuture<'_, NotificationResult, Self::Error>;
}

/// A factory for creating service instances.
///
/// This trait provides methods for creating instances of various services.
/// It's used by the application to get access to the services it needs.
pub trait ServiceFactory: Send + Sync {
    /// Get a calendar service instance.
    fn calendar_service(&self) -> Option<Arc<dyn CalendarService<Error = BoxedError>>>;

    /// Get a payment service instance.
    fn payment_service(&self) -> Option<Arc<dyn PaymentService<Error = BoxedError>>>;

    /// Get a notification service instance.
    fn notification_service(&self) -> Option<Arc<dyn NotificationService<Error = BoxedError>>>;
}

/// Data structures for calendar service operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// The start time of the event.
    pub start_time: String,
    /// The end time of the event.
    pub end_time: String,
    /// The summary or title of the event.
    pub summary: String,
    /// An optional description of the event.
    pub description: Option<String>,
    // The payment method used for the event (e.g., "stripe").
    #[serde(skip)]
    pub payment_method: Option<String>,
    // The payment ID or reference for the event.
    #[serde(skip)]
    pub payment_id: Option<String>,
    // The payment amount in cents.
    #[serde(skip)]
    pub payment_amount: Option<i64>,
    #[serde(skip)]
    pub room_name: Option<String>,
}

/// Represents the result of a calendar event operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventResult {
    /// The ID of the event.
    pub event_id: Option<String>,
    /// The status of the event.
    pub status: String,
}

/// Represents a booked event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookedEvent {
    /// The ID of the event.
    pub event_id: String,
    /// The summary or title of the event.
    pub summary: String,
    /// An optional description of the event.
    pub description: Option<String>,
    /// The start time of the event.
    pub start_time: String,
    /// The end time of the event.
    pub end_time: String,
    /// The status of the event.
    pub status: String,
    /// When the event was created.
    pub created: String,
    /// When the event was last updated.
    pub updated: String,

    pub payment_method: Option<String>,
    pub payment_id: Option<String>,
    pub payment_amount: Option<i64>,
    pub room_name: Option<String>,
}

/// Data structures for payment service operations.
/// Represents the result of a payment intent operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntentResult {
    /// The ID of the payment intent.
    pub id: String,
    /// The status of the payment intent.
    pub status: String,
    /// The amount of the payment intent.
    pub amount: i64,
    /// The currency of the payment intent.
    pub currency: String,
    /// The client secret for the payment intent.
    pub client_secret: Option<String>,
}

/// Represents the result of a refund operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResult {
    /// The ID of the refund.
    pub id: String,
    /// The status of the refund.
    pub status: String,
    /// The amount of the refund.
    pub amount: i64,
    /// The currency of the refund.
    pub currency: String,
}

/// Data structures for notification service operations.
/// Represents the result of a notification operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    /// The ID of the notification.
    pub id: String,
    /// The status of the notification.
    pub status: String,
}
