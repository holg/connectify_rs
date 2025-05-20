# Connectify Architecture Diagrams

This document provides visual representations of the Connectify system architecture and data flows to help developers understand the overall structure and interactions within the system.

## Table of Contents

- [System Architecture](#system-architecture)
- [Component Diagram](#component-diagram)
- [Sequence Diagrams](#sequence-diagrams)
  - [Booking Flow](#booking-flow)
  - [Payment Processing](#payment-processing)
  - [Notification Flow](#notification-flow)
- [Data Flow Diagrams](#data-flow-diagrams)
- [Deployment Architecture](#deployment-architecture)

## System Architecture

The following diagram illustrates the high-level architecture of the Connectify system:

```
+----------------------------------+
|          Client Applications     |
|  (Web, Mobile, External Systems) |
+----------------------------------+
               |
               | HTTP/JSON
               v
+----------------------------------+
|           API Gateway            |
|     (Load Balancer, Routing)     |
+----------------------------------+
               |
               v
+----------------------------------+
|        Connectify Backend        |
|                                  |
|  +-------------+  +------------+ |
|  | API Layer   |->| Handlers   | |
|  +-------------+  +------------+ |
|         |                        |
|         v                        |
|  +-------------+  +------------+ |
|  | Service     |->| Logic      | |
|  | Layer       |  | Layer      | |
|  +-------------+  +------------+ |
|         |                        |
|         v                        |
|  +-------------+                 |
|  | Integration |                 |
|  | Layer       |                 |
|  +-------------+                 |
+----------------------------------+
               |
               v
+----------------------------------+
|        External Services         |
|                                  |
|  +-------------+  +------------+ |
|  | Google      |  | Stripe/    | |
|  | Calendar    |  | Payrexx    | |
|  +-------------+  +------------+ |
|                                  |
|  +-------------+  +------------+ |
|  | Twilio      |  | Other      | |
|  |             |  | Services   | |
|  +-------------+  +------------+ |
+----------------------------------+
```

### Key Components

1. **Client Applications**: Web browsers, mobile apps, or external systems that interact with the Connectify API.

2. **API Gateway**: Routes requests to the appropriate backend services and handles cross-cutting concerns like authentication, rate limiting, and logging.

3. **Connectify Backend**: The core application that processes requests and coordinates interactions with external services.
   - **API Layer**: Handles HTTP requests and responses, routing, and serialization/deserialization.
   - **Handlers Layer**: Processes requests, validates input, and coordinates service calls.
   - **Service Layer**: Contains the business logic for each service domain.
   - **Logic Layer**: Implements the core algorithms and business rules.
   - **Integration Layer**: Communicates with external services.

4. **External Services**: Third-party services that Connectify integrates with, such as Google Calendar, Stripe, Payrexx, and Twilio.

## Component Diagram

The following diagram shows the main components of the Connectify system and their relationships:

```
+------------------+     +------------------+     +------------------+
| Calendar Service |     | Payment Service  |     | Notification     |
|                  |     |                  |     | Service          |
| +-------------+  |     | +-------------+  |     | +-------------+  |
| | Google      |  |     | | Stripe      |  |     | | Twilio      |  |
| | Calendar    |<-|-----|->| Integration |  |     | | Integration |  |
| | Integration |  |     | +-------------+  |     | +-------------+  |
| +-------------+  |     |        |         |     +------------------+
+------------------+     |        v         |              ^
         ^               | +-------------+  |              |
         |               | | Payrexx     |  |              |
         |               | | Integration |  |              |
         |               | +-------------+  |              |
         |               +------------------+              |
         |                        ^                        |
         |                        |                        |
         v                        v                        v
+----------------------------------------------------------+
|                    Service Factory                       |
+----------------------------------------------------------+
                              ^
                              |
                              v
+----------------------------------------------------------+
|                      App State                           |
+----------------------------------------------------------+
                              ^
                              |
                              v
+----------------------------------------------------------+
|                      API Routes                          |
+----------------------------------------------------------+
                              ^
                              |
                              v
+----------------------------------------------------------+
|                    HTTP Handlers                         |
+----------------------------------------------------------+
```

### Key Components

1. **Calendar Service**: Manages calendar events, availability, and bookings.
   - **Google Calendar Integration**: Communicates with the Google Calendar API.

2. **Payment Service**: Processes payments and manages payment-related operations.
   - **Stripe Integration**: Communicates with the Stripe API.
   - **Payrexx Integration**: Communicates with the Payrexx API.

3. **Notification Service**: Sends notifications via email and SMS.
   - **Twilio Integration**: Communicates with the Twilio API.

4. **Service Factory**: Creates and provides access to service instances.

5. **App State**: Manages the application state and provides access to services.

6. **API Routes**: Defines the API endpoints and routes requests to handlers.

7. **HTTP Handlers**: Processes HTTP requests and returns responses.

## Sequence Diagrams

### Booking Flow

The following sequence diagram illustrates the booking flow in Connectify:

```
+--------+    +--------+    +--------+    +--------+    +--------+
| Client |    | API    |    | Calendar|    | Payment |    | Notif. |
|        |    | Handler|    | Service |    | Service |    | Service|
+--------+    +--------+    +--------+    +--------+    +--------+
    |              |            |              |             |
    | GET /availability         |              |             |
    |------------->|            |              |             |
    |              | get_busy_times            |             |
    |              |----------->|              |             |
    |              |            |              |             |
    |              | calculate_available_slots |             |
    |              |----------->|              |             |
    |              |            |              |             |
    | Available slots           |              |             |
    |<-------------|            |              |             |
    |              |            |              |             |
    | POST /book   |            |              |             |
    |------------->|            |              |             |
    |              | create_payment_intent     |             |
    |              |---------------------------->            |
    |              |            |              |             |
    |              | create_event              |             |
    |              |----------->|              |             |
    |              |            |              |             |
    |              | send_notification         |             |
    |              |---------------------------------------->|
    |              |            |              |             |
    | Booking confirmation      |              |             |
    |<-------------|            |              |             |
    |              |            |              |             |
```

### Payment Processing

The following sequence diagram illustrates the payment processing flow in Connectify:

```
+--------+    +--------+    +--------+    +--------+    +--------+
| Client |    | API    |    | Payment |    | Stripe  |    | Fulfill.|
|        |    | Handler|    | Service |    | API     |    | Service |
+--------+    +--------+    +--------+    +--------+    +--------+
    |              |            |              |             |
    | POST /create-checkout-session            |             |
    |------------->|            |              |             |
    |              | create_payment_intent     |             |
    |              |----------->|              |             |
    |              |            | create_checkout_session    |
    |              |            |------------->|             |
    |              |            |              |             |
    |              |            | session_id, checkout_url   |
    |              |            |<-------------|             |
    |              |            |              |             |
    | session_id, checkout_url  |              |             |
    |<-------------|            |              |             |
    |              |            |              |             |
    | Redirect to checkout_url  |              |             |
    |----------------------------------------->|             |
    |              |            |              |             |
    | Complete payment          |              |             |
    |----------------------------------------->|             |
    |              |            |              |             |
    | Redirect to success_url   |              |             |
    |<-----------------------------------------|             |
    |              |            |              |             |
    | Stripe webhook (payment_intent.succeeded)|             |
    |              |<-----------------------------|          |
    |              |            |              |             |
    |              | fulfill_payment           |             |
    |              |---------------------------------------->|
    |              |            |              |             |
```

### Notification Flow

The following sequence diagram illustrates the notification flow in Connectify:

```
+--------+    +--------+    +--------+    +--------+
| Client |    | API    |    | Notif. |    | Twilio |
|        |    | Handler|    | Service|    | API    |
+--------+    +--------+    +--------+    +--------+
    |              |            |              |
    | POST /send-email          |              |
    |------------->|            |              |
    |              | send_email |              |
    |              |----------->|              |
    |              |            | send_email   |
    |              |            |------------->|
    |              |            |              |
    |              |            | email_id     |
    |              |            |<-------------|
    |              |            |              |
    | notification_id           |              |
    |<-------------|            |              |
    |              |            |              |
    | POST /send-sms            |              |
    |------------->|            |              |
    |              | send_sms   |              |
    |              |----------->|              |
    |              |            | send_sms     |
    |              |            |------------->|
    |              |            |              |
    |              |            | sms_id       |
    |              |            |<-------------|
    |              |            |              |
    | notification_id           |              |
    |<-------------|            |              |
    |              |            |              |
```

## Data Flow Diagrams

### Calendar Booking Flow

```
+-------------+     +-------------+     +-------------+
| Client      |     | Connectify  |     | Google      |
| Application |---->| Backend     |---->| Calendar    |
|             |     |             |     | API         |
+-------------+     +-------------+     +-------------+
      |                    |                   |
      v                    v                   v
+-------------+     +-------------+     +-------------+
| User        |     | Booking     |     | Calendar    |
| Input       |     | Data        |     | Event       |
| - Date range|     | - Start time|     | - Start time|
| - Duration  |     | - End time  |     | - End time  |
| - Details   |     | - Summary   |     | - Summary   |
+-------------+     | - Details   |     | - Details   |
                    +-------------+     +-------------+
```

### Payment Processing Flow

```
+-------------+     +-------------+     +-------------+
| Client      |     | Connectify  |     | Payment     |
| Application |---->| Backend     |---->| Gateway     |
|             |     |             |     | (Stripe)    |
+-------------+     +-------------+     +-------------+
      |                    |                   |
      v                    v                   v
+-------------+     +-------------+     +-------------+
| User        |     | Payment     |     | Payment     |
| Input       |     | Intent      |     | Processing  |
| - Amount    |     | - Amount    |     | - Charge    |
| - Currency  |     | - Currency  |     | - Capture   |
| - Details   |     | - Metadata  |     | - Refund    |
+-------------+     +-------------+     +-------------+
                          |
                          v
                    +-------------+
                    | Fulfillment |
                    | - Booking   |
                    | - Notif.    |
                    +-------------+
```

## Deployment Architecture

The following diagram illustrates the deployment architecture of Connectify:

```
+----------------------------------+
|         Load Balancer            |
+----------------------------------+
               |
               v
+----------------------------------+
|      Kubernetes Cluster          |
|                                  |
| +-------------+  +------------+  |
| | Connectify  |  | Connectify |  |
| | Backend Pod |  | Backend Pod|  |
| +-------------+  +------------+  |
|        |               |         |
|        v               v         |
| +----------------------------------+
| |     Shared Volume (Config)       |
| +----------------------------------+
|                                    |
| +----------------------------------+
| |     Kubernetes Secrets           |
| +----------------------------------+
+------------------------------------+
               |
               v
+----------------------------------+
|        External Services         |
|                                  |
|  +-------------+  +------------+ |
|  | Google      |  | Stripe/    | |
|  | Calendar    |  | Payrexx    | |
|  +-------------+  +------------+ |
|                                  |
|  +-------------+  +------------+ |
|  | Twilio      |  | Other      | |
|  |             |  | Services   | |
|  +-------------+  +------------+ |
+----------------------------------+
```

### Key Components

1. **Load Balancer**: Distributes incoming traffic across multiple Connectify backend instances.

2. **Kubernetes Cluster**: Orchestrates the deployment and scaling of Connectify backend pods.
   - **Connectify Backend Pods**: Instances of the Connectify backend application.
   - **Shared Volume (Config)**: Stores configuration files shared across all pods.
   - **Kubernetes Secrets**: Securely stores sensitive information like API keys and credentials.

3. **External Services**: Third-party services that Connectify integrates with.