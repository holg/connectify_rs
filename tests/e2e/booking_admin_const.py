from common_const import get_var_from_env
url_admin_booking = get_var_from_env("url_admin_booking", "/admin/buchungen.html")
url_admin_booking_api = get_var_from_env("url_admin_booking", "/api/admin/bookings")
h1_booking_admin = get_var_from_env("h1_success", "Buchungen verwalten")
load_bookings_id = get_var_from_env("load_bookings_id", "load-bookings")
admin_booking_list_id = get_var_from_env("admin_booking_list_id", "booking-list")
button_confirm_delete_id = get_var_from_env("button_confirm_delete_id", "confirm-delete")
button_delete_text = get_var_from_env("button_delete_text", "Löschen")