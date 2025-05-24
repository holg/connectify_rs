import re
import datetime
from common_const import get_var_from_env
now = datetime.datetime.now()
month_now = now.month
year_now = now.year

stripe_url = re.compile(r"checkout\.stripe\.com")
stripe_iframe_name = re.compile(r"i\.stripe\.com")
stripe_form_name = "PaymentForm-form"
stripe_card_email_id = "email"
stripe_card_number_id = "cardNumber"
stripe_card_expiry_id = "cardExpiry"
stripe_card_cvc_id = "cardCvc"
stripe_card_billing_id = "billingName"
stripe_card_billing_county_id = "billingCountry"
stipe_pay_button_class="SubmitButton"
stripe_adhoc_text = "Adhoc Session starten"
stripe_adhoc_pay_button = get_var_from_env("stripe_adhoc_pay_button", "Verfügbarkeit prüfen & Bezahlen")
stripe_card_test_number = get_var_from_env("stripe_card_test_number", "1231231231231231")
stripe_card_test_expiry = get_var_from_env("stripe_card_test_expiry", f"{month_now:02d} / {year_now % 100}")
stripe_card_test_cvc = get_var_from_env("stripe_card_test_cvc", "123")
stripe_card_test_billing = get_var_from_env("stripe_card_test_billing", "Test Tester")
stripe_card_test_email = get_var_from_env("stripe_card_test_email", "test@example.com")
stripe_success_url = get_var_from_env("stripe_success_url", "https://example.com/success")
