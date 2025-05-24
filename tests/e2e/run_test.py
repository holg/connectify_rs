#!/usr/bin/env python3
"""
Test runner script for E2E booking tests.

This script provides a command-line interface to run the E2E tests for adhoc and gcal booking flows.
It offers options to run specific tests and handles errors gracefully.

Usage:
    python run_test.py [--adhoc] [--gcal] [--admin] [--headless]

Options:
    --adhoc     Run only the adhoc booking test
    --gcal      Run only the gcal booking test
    --admin     Run only the admin bookings test
    --headless  Run tests in headless mode (no browser UI)

If no test options are specified, all tests will be run.
"""

import argparse
import sys
import traceback
from datetime import datetime

from test_gcal_booking import test_gcal_booking_flow
from test_adhoc_booking import test_adhoc_booking_flow
from test_admin_delete_bookings import test_delete_bookings_flow

def run_test(test_func, headless=False):
    """
    Run a test function with proper error handling and timing.
    
    Args:
        test_func: Function to run the test
        headless: Whether to run in headless mode
        
    Returns:
        True if test passed, False if it failed
    """
    test_name = test_func.__name__
    print(f"\n{'=' * 60}")
    print(f"RUNNING TEST: {test_name}")
    print(f"Start time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'=' * 60}")
    
    start_time = datetime.now()
    try:
        # Pass headless parameter if the test function accepts it
        test_func(headless=headless)
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        print(f"\n{'=' * 60}")
        print(f"TEST PASSED: {test_name}")
        print(f"Duration: {duration:.2f} seconds")
        print(f"{'=' * 60}")
        return True
    except Exception as e:
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        print(f"\n{'=' * 60}")
        print(f"TEST FAILED: {test_name}")
        print(f"Duration: {duration:.2f} seconds")
        print(f"Error: {e}")
        print("\nTraceback:")
        traceback.print_exc()
        print(f"{'=' * 60}")
        return False

def main():
    """
    Main function to parse arguments and run tests.
    """
    parser = argparse.ArgumentParser(description="Run E2E booking tests")
    parser.add_argument("--adhoc", action="store_true", help="Run adhoc booking test")
    parser.add_argument("--gcal", action="store_true", help="Run gcal booking test")
    parser.add_argument("--admin", action="store_true", help="Run admin bookings test")
    parser.add_argument("--headless", action="store_true", help="Run tests in headless mode")
    
    args = parser.parse_args()
    
    # If no test is specified, run all tests
    run_all = not (args.adhoc or args.gcal or args.admin)
    
    print(f"Test Runner - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Running in {'headless' if args.headless else 'GUI'} mode")
    
    failed_tests = []
    passed_tests = []
    
    # Run GCal booking test if specified or if running all tests
    if args.gcal or run_all:
        # Modify the test function to accept a headless parameter
        def run_gcal_test(headless=False):
            # Call the original function with the headless parameter
            test_gcal_booking_flow(headless=headless)
        
        if run_test(run_gcal_test, headless=args.headless):
            passed_tests.append("gcal_booking")
        else:
            failed_tests.append("gcal_booking")
    
    # Run Adhoc booking test if specified or if running all tests
    if args.adhoc or run_all:
        # Modify the test function to accept a headless parameter
        def run_adhoc_test(headless=False):
            # Call the original function with the headless parameter
            test_adhoc_booking_flow(headless=headless)
        
        if run_test(run_adhoc_test, headless=args.headless):
            passed_tests.append("adhoc_booking")
        else:
            failed_tests.append("adhoc_booking")
    
    # Run Admin bookings test if specified or if running all tests
    # Run this AFTER the booking tests to have bookings to delete
    if args.admin or run_all:
        # Modify the test function to accept a headless parameter
        def run_admin_test(headless=False):
            # Call the original function with the headless parameter
            test_delete_bookings_flow(headless=headless)
        
        if run_test(run_admin_test, headless=args.headless):
            passed_tests.append("admin_bookings")
        else:
            failed_tests.append("admin_bookings")
    
    # Print summary
    print("\n\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    print(f"Passed: {len(passed_tests)} tests")
    for test in passed_tests:
        print(f"  ✅ {test}")
    
    print(f"\nFailed: {len(failed_tests)} tests")
    for test in failed_tests:
        print(f"  ❌ {test}")
    
    print("=" * 60)
    
    # Return exit code based on test results
    return 1 if failed_tests else 0

if __name__ == "__main__":
    sys.exit(main())