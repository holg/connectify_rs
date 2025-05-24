from common_const import get_var_from_env
viewport_size = {"width":1920, "height":1080}
BASE_URL = get_var_from_env("TEST_BASE_URL", "http://localhost:3000")
HTTP_AUTH_USERNAME = get_var_from_env("HTTP_AUTH_USERNAME", "admin")
HTTP_AUTH_PASSWORD = get_var_from_env("HTTP_AUTH_PASSWORD", "admin123")
if HTTP_AUTH_USERNAME and HTTP_AUTH_PASSWORD:
    http_credentials = {
        'username': HTTP_AUTH_USERNAME,
        'password': HTTP_AUTH_PASSWORD
    }
    print("HTTP Basic Auth credentials will be used.")

    # Create context, passing http_credentials if they exist
context_options = {}
playwright_default_time_out = 15000
if http_credentials:
    context_options['http_credentials'] = http_credentials
