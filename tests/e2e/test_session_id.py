import os
import re
import shutil
import subprocess
from time import sleep
import datetime
from pathlib import Path

from playwright.sync_api import Page, Browser, BrowserContext, sync_playwright

from stripe_test_const import *
from playwright_const import *
from success_const import *
from webcam_const import *

# Simple module-level functions that can be imported without dependencies

def wait_for_api_request(page, url_pattern, method="GET", timeout=30000):
    """
    Wait for a specific API request to complete.
    
    Args:
        page: Playwright page object
        url_pattern: URL pattern to match (string or regex)
        method: HTTP method (GET, POST, etc.)
        timeout: Timeout in milliseconds
        
    Returns:
        The response object from the API request
    """
    print(f"Waiting for {method} request to {url_pattern}")
    
    # Create a future to store the response
    request_event = {}
    
    # Set up the listener before triggering the action
    def handle_request(request):
        if (isinstance(url_pattern, str) and url_pattern in request.url) or \
           (hasattr(url_pattern, "search") and url_pattern.search(request.url)) and \
           request.method == method:
            print(f"Detected matching request: {request.method} {request.url}")
            request_event["request"] = request
            
            # Also capture the response
            response_future = request.response()
            request_event["response_future"] = response_future
    
    # Listen for requests
    page.on("request", handle_request)
    
    # Wait for the request to be detected
    start_time = datetime.datetime.now()
    while "request" not in request_event:
        if (datetime.datetime.now() - start_time).total_seconds() * 1000 > timeout:
            page.remove_listener("request", handle_request)
            raise TimeoutError(f"Timed out waiting for {method} request to {url_pattern}")
        sleep(0.1)
    
    # Wait for the response to be available
    if "response_future" in request_event:
        try:
            response = request_event["response_future"].value
            request_event["response"] = response
            print(f"Request completed with status: {response.status}")
        except Exception as e:
            print(f"Error getting response: {e}")
    
    # Remove the listener
    page.remove_listener("request", handle_request)
    
    return request_event.get("response")

def wait_for_network_idle(page, timeout=5000, max_inflight_requests=0):
    """
    Wait for network to be idle (no in-flight requests).
    
    Args:
        page: Playwright page object
        timeout: Timeout in milliseconds for how long to wait for idle
        max_inflight_requests: Maximum number of allowed in-flight requests
        
    Returns:
        True if network became idle, False otherwise
    """
    print(f"Waiting for network to be idle (max {max_inflight_requests} in-flight requests)")
    
    # Track in-flight requests
    inflight_requests = set()
    
    def on_request(request):
        inflight_requests.add(request)
        print(f"New request: {request.method} {request.url} ({len(inflight_requests)} in flight)")
    
    def on_response(response):
        request = response.request
        if request in inflight_requests:
            inflight_requests.remove(request)
        print(f"Request completed: {request.method} {request.url} ({len(inflight_requests)} in flight)")
    
    def on_request_failed(request):
        if request in inflight_requests:
            inflight_requests.remove(request)
        print(f"Request failed: {request.method} {request.url} ({len(inflight_requests)} in flight)")
    
    # Set up listeners
    page.on("request", on_request)
    page.on("response", on_response)
    page.on("requestfailed", on_request_failed)
    
    try:
        # Wait for network to be idle
        start_time = datetime.datetime.now()
        while len(inflight_requests) > max_inflight_requests:
            if (datetime.datetime.now() - start_time).total_seconds() * 1000 > timeout:
                print(f"Timed out waiting for network idle. Still have {len(inflight_requests)} requests in flight.")
                return False
            sleep(0.1)
        
        return True
    finally:
        # Clean up listeners
        page.remove_listener("request", on_request)
        page.remove_listener("response", on_response)
        page.remove_listener("requestfailed", on_request_failed)

def wait_and_click(page, selector, wait_for_network=True, timeout=30000):
    """
    Wait for an element to be visible, click it, and optionally wait for network to be idle.
    
    Args:
        page: Playwright page object
        selector: Element selector
        wait_for_network: Whether to wait for network to be idle after clicking
        timeout: Timeout in milliseconds
        
    Returns:
        The element that was clicked
    """
    print(f"Waiting for element: {selector}")
    page.wait_for_selector(selector, state="visible", timeout=timeout)
    element = page.locator(selector).first
    
    print(f"Clicking element: {selector}")
    element.click()
    
    if wait_for_network:
        wait_for_network_idle(page)
    
    return element

def setup_browser(headless=False, record_video=True, test_name=None):
    """
    Create a browser session with appropriate settings.
    
    Args:
        headless: Whether to run in headless mode
        record_video: Whether to record video
        test_name: Name of the test to include in recording filename
        
    Returns:
        Tuple of (playwright, page, context, browser, recording_info)
        where recording_info is a tuple of (temp_dir, mp4_filename) or None if not recording
    """
    playwright = sync_playwright().start()
    browser = playwright.chromium.launch(headless=headless)
    
    # Configure context options
    context_opts = context_options.copy()
    context_opts['permissions'] = ['camera', 'microphone']
    
    recording_info = None
    if record_video:
        # Set up recording paths
        recording_dir = Path(os.path.dirname(__file__)) / "recordings"
        recording_dir.mkdir(exist_ok=True)
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Use test_name if provided, otherwise use generic "test"
        prefix = test_name if test_name else "test"
        
        temp_dir = str(recording_dir / f"temp_{timestamp}")
        mp4_filename = str(recording_dir / f"{prefix}_{timestamp}.mp4")
        
        os.makedirs(temp_dir, exist_ok=True)
        
        # Add recording configuration
        context_opts['record_video_dir'] = temp_dir
        context_opts['record_video_size'] = {"width": 1280, "height": 720}
        
        recording_info = (temp_dir, mp4_filename)
        print(f"Recording video to: {temp_dir}")
        print(f"Final MP4 will be: {mp4_filename}")
    
    # Create context and page
    context = browser.new_context(**context_opts)
    context.set_default_timeout(playwright_default_time_out)
    
    page = context.new_page()
    page.set_default_timeout(playwright_default_time_out)
    page.set_default_navigation_timeout(playwright_default_time_out)
    page.set_viewport_size(viewport_size)
    
    return playwright, page, context, browser, recording_info

def cleanup_browser(playwright, context, browser, recording_info):
    """
    Clean up browser resources and convert video if needed.
    
    Args:
        playwright: The playwright instance
        context: Browser context
        browser: Browser instance
        recording_info: Tuple of (temp_dir, mp4_filename) or None
    """
    # Close browser
    context.close()
    browser.close()
    
    # Convert video if recording was enabled
    if recording_info:
        temp_dir, mp4_filename = recording_info
        try:
            # Find the WebM file
            webm_files = [f for f in os.listdir(temp_dir) if f.endswith('.webm')]
            if webm_files:
                webm_path = os.path.join(temp_dir, webm_files[0])
                print(f"Converting {webm_path} to MP4...")
                
                # Use ffmpeg to convert
                result = subprocess.run([
                    'ffmpeg', 
                    '-i', webm_path, 
                    '-c:v', 'libx264',
                    '-preset', 'fast',
                    '-crf', '22',
                    '-c:a', 'aac',
                    '-b:a', '128k',
                    mp4_filename
                ], capture_output=True)
                
                if result.returncode == 0:
                    print(f"Successfully converted to MP4: {mp4_filename}")
                else:
                    print(f"Error converting to MP4: {result.stderr.decode()}")
            else:
                print("No WebM files found in the recording directory")
                
            # Clean up temporary directory
            shutil.rmtree(temp_dir, ignore_errors=True)
        except Exception as e:
            print(f"Error during video conversion: {e}")
    
    # Stop playwright
    playwright.stop()

def verify_success_page(page, session_id):
    """
    Verify all elements on the success page.
    """
    # Check header
    page.wait_for_selector(f"h1:has-text('{h1_success}')")
    success_h1_text = page.query_selector(f"h1:has-text('{h1_success}')")
    assert success_h1_text.text_content() == h1_success, f"Expected '{h1_success}', got '{success_h1_text.text_content()}'"
    
    # Check details
    page.wait_for_selector(f"#{details_id}:has-text('{details_success_text}')")
    details_element = page.query_selector(f"#{details_id}")
    assert details_element.text_content() == details_success_text, f"Expected '{details_success_text}', got '{details_element.text_content()}'"
    
    # Check session ID display
    session_element = page.query_selector(f"#{session_id_display}")
    assert session_element.text_content() == f"{session_id}", f"Expected '{session_id}', got '{session_element.text_content()}'"
    
    # Check room info
    room_info_element = page.query_selector(f"#{room_info_id}")
    # Note: Commented out assertion in original code
    # assert room_info_element.text_content() == f"gcal_{session_id}"

def test_webcam_interface(page, wc_room_name, wc_user_name):
    """
    Test the webcam interface functionality.
    """
    # Wait for webcam page to load
    full_webcam_url_pattern = re.compile(rf".*webcam\.html\?room={re.escape(wc_room_name)}&user={re.escape(wc_user_name)}")
    print(f"Waiting for URL matching pattern: {full_webcam_url_pattern.pattern}")
    page.wait_for_url(full_webcam_url_pattern)
    
    # Check that join button is enabled
    page.wait_for_selector(f"#{button_join_id}:visible:not([disabled])")
    
    # Verify room and user information
    room_name = page.query_selector(f"#{room_name_id}")
    assert room_name.input_value() == wc_room_name, f"Expected '{wc_room_name}', got '{room_name.input_value()}'"
    
    user_name = page.query_selector(f"#{user_name_id}")
    assert user_name.input_value() == stripe_card_test_billing, f"Expected '{stripe_card_test_billing}', got '{user_name.input_value()}'"
    
    # Get control buttons
    button_join = page.query_selector(f"#{button_join_id}")
    button_leave = page.query_selector(f"#{button_leave_id}")
    
    # Join webcam room
    print("Joining the webcam room...")
    button_join.click()
    
    # Verify video element appears
    print("Waiting for video element to appear...")
    page.wait_for_selector("video")
    
    # Stay in room for a while to record
    print("Recording webcam session for 10 seconds...")
    sleep(10)
    
    # Leave the room
    print("Leaving the webcam room...")
    button_leave.click()
    sleep(2)
    
    # Verify video element is gone or hidden
    video_after_leave = page.query_selector("video")
    assert video_after_leave is None or not video_after_leave.is_visible(), "Webcam interface is still visible after leaving"
    
    print(f"Successfully completed webcam test. Room name: {room_name.input_value()}")

def test_session_id(page, session_id):
    """
    Test the session ID validation flow.
    
    Args:
        page: The playwright page
        session_id: Session ID to test
    """
    # Navigate to success page
    page.goto(f"{stripe_success_url}?session_id={session_id}")
    
    # Verify URL and session ID
    url_pattern = re.compile(rf"{stripe_success_url}\?session_id=([^&]*)")
    page.wait_for_url(url_pattern)
    current_url = page.url
    extracted_session_id = re.search(url_pattern, current_url).group(1)
    print(f"Session ID: {extracted_session_id}")
    
    # Verify success page elements
    verify_success_page(page, session_id)
    
    # Navigate to webcam page
    webcam_link_element = page.query_selector(f"#{webcam_link_id}")
    webcam_url = webcam_link_element.get_attribute("href")
    
    # Extract room and user parameters
    webcam_url_pattern = re.compile(rf"/webcam\.html\?room=(.*)&user=(.*)")
    match = re.search(webcam_url_pattern, webcam_url)
    if match:
        wc_room_name = match.group(1)
        wc_user_name = match.group(2)
    else:
        raise ValueError("Could not extract room and user name from webcam URL")
    
    # Navigate to webcam page
    webcam_link_element.click()
    
    # Test webcam functionality
    test_webcam_interface(page, wc_room_name, wc_user_name)

# Direct test execution example
if __name__ == "__main__":
    session_id = "cs_test_a1jIKldw4HPURTO6VAvUdV0g7FfaRv3bTpemmYG2tok3FNXqCMDARJfyfb"
    
    playwright, page, context, browser, recording_info = setup_browser(headless=False, test_name="test_session_id")
    
    try:
        test_session_id(page, session_id)
        print("Test completed successfully")
    except Exception as e:
        print(f"Test failed: {e}")
        raise
    finally:
        cleanup_browser(playwright, context, browser, recording_info)