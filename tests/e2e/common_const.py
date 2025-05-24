import os
import dotenv
from dotenv import load_dotenv
load_dotenv()

def get_var_from_env(var_name, override=None):
    """
    Get environment variable value.
    """
    result = os.getenv(var_name.upper())
    if not result:
        result = globals().get(var_name)
        if override and not result:
            result = override
    return result



