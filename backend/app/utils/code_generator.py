"""Utility functions for generating codes"""
import random
import string


def generate_session_code(length: int = 8) -> str:
    """Generate a random session code"""
    # Use uppercase letters and numbers, excluding similar looking characters
    chars = string.ascii_uppercase + string.digits
    chars = chars.replace('O', '').replace('0', '').replace('I', '').replace('1', '').replace('L', '')
    return ''.join(random.choice(chars) for _ in range(length))
