import re
from time import sleep

from test_session_id import (
    setup_browser, cleanup_browser, test_session_id
)
from stripe_test_const import *
from playwright_const import *
from success_const import *
from webcam_const import *


def test_adhoc_booking_flow(headless=False):
    """
    End-to-end test for the adhoc booking flow:
    1. Navigate to the adhoc booking page
    2. Select a duration and submit the form
    3. Process payment with test credit card
    4. Verify redirection to webcam page
    """
    # Setup browser with shared function
    playwright, page, context, browser, recording_info = setup_browser(headless=headless, test_name="adhoc_booking_flow")

    try:
        # Step 1: Navigate to the adhoc booking page
        page.goto(f"{BASE_URL}/")

        # Wait for the page to load
        page.wait_for_selector(f"Button:has-text('{stripe_adhoc_text}')")

        # Step 2: Select a duration and submit the form
        # Find the duration dropdown or radio buttons and select a duration
        page.click(f"button:has-text('{stripe_adhoc_text}')")  # Adjust selector based on actual UI

        # Click the book now button
        page.click(f"button:has-text('{stripe_adhoc_pay_button}')")

        # Wait for the API call to complete and redirect to Stripe
        page.wait_for_url(stripe_url)

        # Step 3: Fill out the Stripe payment form with test credit card
        # Wait for Stripe iframe to load
        page.wait_for_selector(f".{stripe_form_name}")

        # Get the card iframe
        card_form = page.locator(f".{stripe_form_name}").first

        # Fill out card details
        # email
        card_form.locator(f"#{stripe_card_email_id}").fill(stripe_card_test_email)
        # card number
        card_form.locator(f"#{stripe_card_number_id}").fill(stripe_card_test_number)
        # card expiry date
        card_form.locator(f"#{stripe_card_expiry_id}").fill(stripe_card_test_expiry)
        # card cvc
        card_form.locator(f"#{stripe_card_cvc_id}").fill(stripe_card_test_cvc)
        # card billing Name
        card_form.locator(f"#{stripe_card_billing_id}").fill(stripe_card_test_billing)

        # Submit payment
        page.click("button:has-text('Pay')")

        # Step 4: Wait for redirect to success page
        url_pattern = re.compile(rf"{stripe_success_url}\?session_id=([^&]*)")
        page.wait_for_url(url_pattern)
        current_url = page.url
        session_id = re.search(url_pattern, current_url).group(1)
        print(f"Session ID: {session_id}")

        # Use the shared test_session_id function to continue with webcam testing
        test_session_id(page, session_id)

    finally:
        # Clean up with shared function
        cleanup_browser(playwright, context, browser, recording_info)


if __name__ == "__main__":
    test_adhoc_booking_flow()
