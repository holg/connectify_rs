# End-to-End Tests with Playwright

This directory contains end-to-end tests for the Connectify application using Python and Playwright.

## Prerequisites

- Python 3.8 or higher
- pip (Python package manager)

## Setup

1. Install the required dependencies:

```bash
pip install -r requirements.txt
```

2. Install Playwright browsers:

```bash
playwright install
```

## Running the Tests

To run all the e2e tests:

```bash
pytest -v
```

To run a specific test:

```bash
pytest -v test_adhoc_booking.py
```

## Test Descriptions

### Adhoc Booking Flow Test

The `test_adhoc_booking.py` file contains an end-to-end test for the adhoc booking flow:

1. Navigate to the adhoc booking page
2. Select a duration and submit the form
3. Process payment with test credit card data from the .env file
4. Verify redirection to the webcam page

This test verifies the complete user journey from booking an adhoc session to joining the webcam room.

## Configuration

The tests use environment variables for configuration:

- `TEST_BASE_URL`: The base URL of the application (default: http://localhost:3000)
- `STRIPE_TEST_MASTERCARD`: The test credit card number to use for payments

These variables can be set in the `.env` file at the project root or passed directly when running the tests:

```bash
TEST_BASE_URL=https://staging.example.com pytest -v
```

## Troubleshooting

If you encounter issues with the tests:

1. Make sure the application is running and accessible at the configured base URL
2. Check that the test credit card data in the .env file is valid
3. Verify that the selectors in the test match the actual UI elements
4. Run the tests with the `--headed` flag to see the browser in action:

```bash
pytest -v --headed
```

## Adding New Tests

When adding new e2e tests:

1. Create a new Python file with a name that describes the flow being tested
2. Use the existing tests as a template
3. Follow the pattern of navigating to a page, interacting with elements, and verifying the expected outcome
4. Add documentation for the new test in this README