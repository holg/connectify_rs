import re
from time import sleep

from test_session_id import (
    setup_browser, cleanup_browser, test_session_id, wait_for_api_request
)
from stripe_test_const import *
from playwright_const import *
from booking_admin_const import *


def test_delete_bookings_flow(headless=False):
    """
    End-to-end test for deleting all bookings in the admin interface.
    Handles the case where the page reloads after each deletion and
    also detects when there are no bookings.
    """
    # Setup browser with shared function
    playwright, page, context, browser, recording_info = setup_browser(headless=headless, test_name="delete_bookings_flow")

    try:
        # Navigate to the admin bookings page
        print("Navigating to admin bookings page...")
        page.goto(f"{BASE_URL}{url_admin_booking}")
        
        # Wait for page to load
        page.wait_for_load_state("networkidle")
        print("Page loaded")
        
        # Check if there are any bookings or if the "no bookings" message is displayed
        no_bookings_message = page.locator("p.text-center:text('Keine Buchungen im Zeitraum.')")
        
        if no_bookings_message.is_visible():
            print("No bookings found - nothing to delete")
            return
            
        # Keep deleting until no more bookings are found or the "no bookings" message appears
        deleted_count = 0
        
        while True:
            # Wait for the page to be fully loaded
            page.wait_for_load_state("networkidle")
            
            # Check if the "no bookings" message is visible now
            if page.locator("p.text-center:text('Keine Buchungen im Zeitraum.')").is_visible():
                print(f"All bookings deleted. Total: {deleted_count}")
                break
                
            # Get a fresh reference to the booking list
            booking_list = page.locator(f"#{admin_booking_list_id}")
            
            # Get the first booking item (if any)
            booking_items = booking_list.locator("> *")
            
            # Check if there are any booking items
            if booking_items.count() == 0:
                print(f"No more booking items found. Total deleted: {deleted_count}")
                break
                
            # Get the first booking item
            booking_item = booking_items.first
            
            # Try to find the delete button
            delete_button = booking_item.locator(f"button:has-text('{button_delete_text}')")
            
            # If the button with text isn't found, try finding it by onclick attribute
            if delete_button.count() == 0:
                delete_button = booking_item.locator("button[onclick*='promptDelete']")
            
            # If still no delete button found, log and break
            if delete_button.count() == 0:
                print("No delete button found in the booking item. Stopping.")
                break
                
            # Extract booking details for logging
            try:
                # Try to get ID from onclick attribute
                onclick = delete_button.get_attribute("onclick")
                booking_id = "unknown"
                
                if onclick:
                    import re
                    match = re.search(r"promptDelete\('([^']+)'", onclick)
                    if match:
                        booking_id = match.group(1)
                        
                print(f"Deleting booking with ID: {booking_id}")
            except Exception as e:
                print(f"Error extracting booking ID: {e}")
                print("Continuing with deletion anyway...")
            
            # Set up dialog handler for confirmation dialog
            page.once("dialog", lambda dialog: dialog.accept())
            
            # Click the delete button
            delete_button.click()
            
            # Wait for the confirmation dialog to be handled
            page.wait_for_timeout(500)
            
            # Look for confirm delete button if it appears
            try:
                confirm_delete = page.locator(f"#{button_confirm_delete_id}")
                if confirm_delete.is_visible(timeout=2000):
                    print("Clicking confirm delete button")
                    confirm_delete.click()
            except Exception as e:
                print(f"No confirm button found or error: {e}")
                print("Continuing with test...")
            
            # Wait for the page to reload/update after deletion
            page.wait_for_load_state("networkidle")
            
            # Increment counter
            deleted_count += 1
            print(f"Successfully deleted booking #{deleted_count}")
            
            # Brief pause to ensure everything is settled
            page.wait_for_timeout(1000)
            
        print(f"Test completed. Total bookings deleted: {deleted_count}")

    except Exception as e:
        print(f"Test failed with error: {e}")
        # Take a screenshot on failure
        page.screenshot(path=f"delete_bookings_error_{datetime.datetime.now().strftime('%Y%m%d_%H%M%S')}.png")
        raise
        
    finally:
        # Clean up with shared function
        cleanup_browser(playwright, context, browser, recording_info)

if __name__ == "__main__":
    test_delete_bookings_flow()