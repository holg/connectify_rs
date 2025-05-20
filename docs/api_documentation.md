# Connectify API Documentation

This document provides comprehensive documentation for the Connectify API, including endpoints, request/response formats, and examples.

## Table of Contents

- [Introduction](#introduction)
- [Authentication](#authentication)
- [API Endpoints](#api-endpoints)
  - [Calendar API](#calendar-api)
  - [Payment API](#payment-api)
  - [Notification API](#notification-api)
  - [Fulfillment API](#fulfillment-api)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [Examples](#examples)

## Introduction

The Connectify API is a RESTful API that provides access to various services including calendar management, payment processing, notifications, and fulfillment. The API is designed to be easy to use and follows standard REST conventions.

## Authentication

Authentication is required for all API endpoints. The API uses token-based authentication. To authenticate, include an `Authorization` header in your request with a valid token:

```
Authorization: Bearer your_token_here
```

Tokens can be obtained by calling the authentication endpoint with valid credentials.

## API Endpoints

### Calendar API

The Calendar API provides endpoints for managing calendar events, checking availability, and booking appointments.

#### Get Available Slots

```
GET /api/gcal/availability
```

Query Parameters:
- `start_date` (required): Start date in YYYY-MM-DD format
- `end_date` (required): End date in YYYY-MM-DD format
- `duration_minutes` (required): Duration in minutes

Response:
```json
{
  "slots": [
    {
      "start_time": "2025-05-15T10:00:00Z",
      "end_time": "2025-05-15T11:00:00Z",
      "duration_minutes": 60,
      "price": 7500,
      "currency": "CHF",
      "product_name": "Premium Beratung (60 Min)"
    },
    {
      "start_time": "2025-05-15T14:00:00Z",
      "end_time": "2025-05-15T15:00:00Z",
      "duration_minutes": 60,
      "price": 7500,
      "currency": "CHF",
      "product_name": "Premium Beratung (60 Min)"
    }
  ]
}
```

#### Book a Slot

```
POST /api/gcal/book
```

Request Body:
```json
{
  "start_time": "2025-05-15T10:00:00Z",
  "end_time": "2025-05-15T11:00:00Z",
  "summary": "Consultation with John Doe",
  "description": "Initial consultation to discuss project requirements"
}
```

Response:
```json
{
  "success": true,
  "event_id": "abc123",
  "message": "Appointment booked successfully."
}
```

#### Get Booked Events

```
GET /api/gcal/admin/bookings
```

Query Parameters:
- `start_date` (required): Start date in YYYY-MM-DD format
- `end_date` (required): End date in YYYY-MM-DD format
- `include_cancelled` (optional): Whether to include cancelled events (default: false)

Response:
```json
{
  "events": [
    {
      "event_id": "abc123",
      "summary": "Consultation with John Doe",
      "description": "Initial consultation to discuss project requirements",
      "start_time": "2025-05-15T10:00:00Z",
      "end_time": "2025-05-15T11:00:00Z",
      "status": "confirmed",
      "created": "2025-05-10T14:30:00Z",
      "updated": "2025-05-10T14:30:00Z"
    }
  ]
}
```

#### Cancel a Booking

```
PATCH /api/gcal/admin/mark_cancelled/{event_id}
```

Query Parameters:
- `notify_attendees` (optional): Whether to notify attendees (default: true)

Response:
```json
{
  "success": true,
  "message": "Appointment marked as cancelled successfully."
}
```

#### Delete a Booking

```
DELETE /api/gcal/admin/delete/{event_id}
```

Query Parameters:
- `notify_attendees` (optional): Whether to notify attendees (default: true)

Response:
```json
{
  "success": true,
  "message": "Event deleted successfully."
}
```

### Payment API

The Payment API provides endpoints for processing payments using Stripe or Payrexx.

#### Create Stripe Checkout Session

```
POST /api/stripe/create-checkout-session
```

Request Body:
```json
{
  "product_name_override": "Premium Consultation",
  "amount_override": 7500,
  "currency_override": "CHF",
  "fulfillment_type": "appointment",
  "fulfillment_data": {
    "start_time": "2025-05-15T10:00:00Z",
    "end_time": "2025-05-15T11:00:00Z",
    "summary": "Consultation with John Doe"
  },
  "client_reference_id": "client123"
}
```

Response:
```json
{
  "session_id": "cs_test_abc123",
  "checkout_url": "https://checkout.stripe.com/pay/cs_test_abc123"
}
```

#### Create Payrexx Payment Page

```
POST /api/payrexx/create-payment-page
```

Request Body:
```json
{
  "product_name_override": "Premium Consultation",
  "amount_override": 7500,
  "currency_override": "EUR",
  "fulfillment_type": "appointment",
  "fulfillment_data": {
    "start_time": "2025-05-15T10:00:00Z",
    "end_time": "2025-05-15T11:00:00Z",
    "summary": "Consultation with John Doe"
  },
  "client_reference_id": "client123"
}
```

Response:
```json
{
  "payment_page_id": "123456",
  "payment_url": "https://payrexx.com/pay/123456"
}
```

### Notification API

The Notification API provides endpoints for sending notifications via email or SMS using Twilio.

#### Send Email

```
POST /api/twilio/send-email
```

Request Body:
```json
{
  "to": "recipient@example.com",
  "subject": "Appointment Confirmation",
  "body": "Your appointment has been confirmed for May 15, 2025 at 10:00 AM.",
  "is_html": false
}
```

Response:
```json
{
  "id": "abc123",
  "status": "sent"
}
```

#### Send SMS

```
POST /api/twilio/send-sms
```

Request Body:
```json
{
  "to": "+41791234567",
  "body": "Your appointment has been confirmed for May 15, 2025 at 10:00 AM."
}
```

Response:
```json
{
  "id": "abc123",
  "status": "sent"
}
```

### Fulfillment API

The Fulfillment API provides endpoints for fulfilling orders and bookings.

#### Fulfill Booking

```
POST /api/fulfillment/gcal-booking
```

Request Body:
```json
{
  "start_time": "2025-05-15T10:00:00Z",
  "end_time": "2025-05-15T11:00:00Z",
  "summary": "Consultation with John Doe",
  "description": "Initial consultation to discuss project requirements",
  "payment_intent_id": "pi_abc123"
}
```

Response:
```json
{
  "success": true,
  "event_id": "abc123",
  "message": "Booking fulfilled successfully."
}
```

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of a request. In case of an error, the response body will contain a JSON object with an error message:

```json
{
  "error": {
    "code": "invalid_request",
    "message": "The request was invalid. Please check your parameters and try again."
  }
}
```

Common error codes:
- `400 Bad Request`: The request was invalid.
- `401 Unauthorized`: Authentication failed.
- `403 Forbidden`: The authenticated user does not have permission to access the requested resource.
- `404 Not Found`: The requested resource was not found.
- `409 Conflict`: The request could not be completed due to a conflict with the current state of the resource.
- `500 Internal Server Error`: An error occurred on the server.

## Rate Limiting

The API implements rate limiting to prevent abuse. The rate limits are as follows:
- 100 requests per minute per IP address
- 1000 requests per hour per IP address

If you exceed the rate limit, you will receive a `429 Too Many Requests` response with a `Retry-After` header indicating how many seconds to wait before making another request.

## Examples

### Booking a Slot

This example shows how to book a slot using the Calendar API:

```javascript
// Get available slots
const response = await fetch('https://api.connectify.com/api/gcal/availability?start_date=2025-05-15&end_date=2025-05-15&duration_minutes=60', {
  headers: {
    'Authorization': 'Bearer your_token_here'
  }
});

const availableSlots = await response.json();

// Book the first available slot
const slot = availableSlots.slots[0];
const bookingResponse = await fetch('https://api.connectify.com/api/gcal/book', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer your_token_here',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    start_time: slot.start_time,
    end_time: slot.end_time,
    summary: 'Consultation with John Doe',
    description: 'Initial consultation to discuss project requirements'
  })
});

const booking = await bookingResponse.json();
console.log(booking);
```

### Processing a Payment

This example shows how to create a Stripe checkout session:

```javascript
const response = await fetch('https://api.connectify.com/api/stripe/create-checkout-session', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer your_token_here',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    product_name_override: 'Premium Consultation',
    amount_override: 7500,
    currency_override: 'CHF',
    fulfillment_type: 'appointment',
    fulfillment_data: {
      start_time: '2025-05-15T10:00:00Z',
      end_time: '2025-05-15T11:00:00Z',
      summary: 'Consultation with John Doe'
    },
    client_reference_id: 'client123'
  })
});

const session = await response.json();
window.location.href = session.checkout_url;
```