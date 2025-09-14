# Frappe Python API: Utils — Practical Cheat-Sheet

> Import style
>
> ```py
> from frappe.utils import (
>     now, getdate, today, add_to_date, date_diff, days_diff, month_diff,
>     pretty_date, format_duration, comma_and, money_in_words,
>     validate_json_string, random_string, unique, get_abbr,
>     validate_url, validate_email_address, validate_phone_number
> )
> from frappe.utils.pdf import get_pdf
> import frappe  # for frappe.cache(), frappe.sendmail()
> ```
>
> Utilities live under `frappe.utils` (and submodules like `frappe.utils.pdf`).

---

## Date & Time

### `now() -> str`

Current datetime in `"YYYY-MM-DD HH:MM:SS(.ffffff)"`.

```py
ts = now()  # e.g. '2021-05-25 06:38:52.242515'
```

### `getdate(string_date: str | None = None) -> datetime.date`

Parses `'YYYY-MM-DD'` to `date`. `None` → today. Raises on invalid string.

```py
d1 = getdate()            # today as date
d2 = getdate('2000-03-18')  # datetime.date(2000, 3, 18)
```

### `today() -> str`

Current date as `'YYYY-MM-DD'`.

```py
ymd = today()
```

### `add_to_date(date_or_str, *, years=0, months=0, weeks=0, days=0, hours=0, minutes=0, seconds=0, as_string=False, as_datetime=False)`

Add a delta to a date/datetime (or pass `None` for “now”).

- If `as_string=True` and `as_datetime=True` → returns datetime string.
- If `as_string=True` and `as_datetime=False` → date string.

```py
in_10_days = add_to_date(getdate(), days=10, as_string=True)
in_2_months = add_to_date(now(), months=2)            # datetime
```

**Gotcha:** Month arithmetic follows calendar math (not 30-day blocks).

### `date_diff(later, earlier) -> int` and `days_diff(later, earlier) -> int`

Difference in days (integer).

```py
delta = date_diff(add_to_date(today(), days=10), today())  # 10
```

### `month_diff(later, earlier) -> int`

Difference in whole months.

```py
m = month_diff(add_to_date("2024-07-01", days=60), "2024-07-01")  # 2
```

### `pretty_date(iso_datetime_str) -> str`

Human-friendly “x minutes/hours/days ago”.

```py
pretty = pretty_date(now())  # 'just now'
```

### `format_duration(seconds: float, hide_days: bool = False) -> str`

Formats seconds to `11d 13h 46m 40s` etc.

```py
format_duration(10000)            # '2h 46m 40s'
format_duration(1_000_000, True)  # '277h 46m 40s'
```

---

## Strings & Lists

### `comma_and(items: list | tuple | any, add_quotes: bool = True) -> str`

Join with commas and localized “and”.

```py
comma_and(['Apple', 'Ball', 'Cat'], add_quotes=False)  # 'Apple, Ball and Cat'
```

> Sibling: `comma_or` (same idea with “or”).

### `money_in_words(number: float, main_currency: str | None = None, fraction_currency: str | None = None) -> str`

Converts amount to words with currency.

```py
money_in_words(900.50, 'USD')               # 'USD Nine Hundred and Fifty Centavo only.'
money_in_words(900.50, 'USD', 'Cents')      # 'USD Nine Hundred and Fifty Cents only.'
```

**Note:** Defaults come from site settings; INR fallback if none.

### `validate_json_string(s: str) -> None`

Valid → no exception. Invalid → raises `frappe.ValidationError`.

```py
try:
    validate_json_string('invalid json')
except frappe.ValidationError:
    ...
```

### `random_string(length: int) -> str`

Random alphanumeric string of given length.

```py
token = random_string(40)
```

### `unique(seq: Iterable[T]) -> list[T]`

Order-preserving de-duplication.

```py
unique([1,2,3,1,1])  # [1,2,3]
```

### `get_abbr(s: str, max_len: int = 2) -> str`

Initials/abbr for names, useful for avatars.

```py
get_abbr('Coca Cola Company')      # 'CC'
get_abbr('Mohammad Hussain', 3)    # 'MHN'
```

---

## Validation

### `validate_url(txt: str, throw: bool=False, valid_schemes: str | Iterable[str] | None=None) -> bool`

Returns `True/False`. If `throw=True`, raises `ValidationError` on invalid.

```py
validate_url('https://google.com')           # True
validate_url('https://google.com', throw=True)  # ok
```

**Tip:** Use `valid_schemes={'http','https'}` to restrict.

### `validate_email_address(email_str: str, throw: bool=False) -> str`

Returns a single email or comma-separated list of **valid** emails found; empty string if none. If `throw=True`, raises `frappe.InvalidEmailAddressError` when none found.

```py
validate_email_address('a@x.com, b@x.com')  # 'a@x.com, b@x.com'
```

### `validate_phone_number(phone: str, throw: bool=False) -> bool`

`True/False`. If invalid and `throw=True`, raises `frappe.InvalidPhoneNumberError`.

```py
validate_phone_number('+91-75385837')  # True
```

---

## PDF

### `frappe.utils.pdf.get_pdf(html: str, options: dict | None=None, output: PdfFileWriter | None=None) -> bytes | PdfFileWriter`

Renders HTML to PDF using `pdfkit`/`PyPDF2`.

- If `output` (a `PdfFileWriter`) is given → pages are appended and the `PdfFileWriter` is returned.
- Else → returns `bytes`.

```py
from frappe.utils.pdf import get_pdf

@frappe.whitelist(allow_guest=True)
def invoice_pdf():
    html = '<h1>Invoice</h1>'
    frappe.local.response.filename = 'invoice.pdf'
    frappe.local.response.filecontent = get_pdf(html)
    frappe.local.response.type = 'pdf'
```

**Gotcha:** wkhtmltopdf must be available in your environment; CSS support is the wkhtmltopdf subset.

---

## Caching (Redis)

### `frappe.cache() -> RedisWrapper`

Low-level Redis client for transient key/values.

```py
cache = frappe.cache()
cache.set('greeting', 'hello')
cache.get('greeting')  # b'hello'
```

**Best practice:** Namespace keys (e.g., `myapp:feature:x`) and set TTLs for ephemeral data.

---

## Email

### `frappe.sendmail(recipients=[], sender="", subject="No Subject", message="No Message", as_markdown=False, template=None, args=None, **kwargs) -> None`

High-level mailer using user/global outgoing account.
Common kwargs: `attachments=[{"file_url": "/files/x.png"}]`, `header="..."`, etc.

```py
frappe.sendmail(
    recipients=["user@example.com"],
    subject="Reminder",
    template="birthday_reminder",
    args={"reminder_text": "Hi!", "birthday_persons": [], "message": "Enjoy!"},
    as_markdown=False,
)
```

**Gotcha:** When using templates, ensure your Jinja lives in `templates/emails/`. Attachments use `File.file_url`.

---

## File Locks (synchronization)

### `from frappe.utils.synchronization import filelock`

Named lock to protect critical sections across processes.

```py
from frappe.utils.synchronization import filelock

def write_config(cfg, path):
    with filelock("my_config_lock"):
        with open(path, "w") as f:
            f.write(cfg)
```

**Use cases:** Avoid race conditions in cron/jobs while writing to the same file.

---

## Common Recipes

**1) Add “N business days” to today and email a reminder**

```py
from datetime import timedelta
from frappe.utils import getdate, add_to_date
import frappe

def remind_in_n_days(n=5, email="ops@example.com"):
    # business-day add (simple): add n days; skip weekends manually if needed
    target = add_to_date(getdate(), days=n, as_string=True)
    frappe.sendmail(recipients=[email], subject="Reminder", message=f"Due on {target}")
```

(Uses `add_to_date`, `getdate`, `sendmail`.)

**2) Validate a user’s contact fields**

```py
from frappe.utils import validate_email_address, validate_phone_number, validate_url

def validate_contact(email, phone, website=None):
    valid_emails = validate_email_address(email, throw=True)
    if not validate_phone_number(phone):
        frappe.throw("Phone number is invalid")
    if website is not None and not validate_url(website):
        frappe.throw("Website URL is invalid")
    return valid_emails
```

**3) Generate a PDF bytes blob and attach to a DocType**

```py
from frappe.utils.pdf import get_pdf

def attach_pdf_to_doc(doc):
    html = f"<h1>{doc.title}</h1>"
    pdf_bytes = get_pdf(html)
    # create a File document with attached content...
```

**4) Cache an expensive lookup**

```py
import json, frappe

def get_cached_settings():
    cache = frappe.cache()
    key = "myapp:settings:v1"
    cached = cache.get(key)
    if cached:
        return json.loads(cached)
    # fallback: read from DocType or config file
    data = {"feature": True}
    cache.set(key, json.dumps(data))
    return data
```

---

## Pitfalls & Tips

- **Timezones**: `now()`/`today()` return strings; when mixing with Python `datetime`, normalize to aware datetimes if you rely on TZ math. (The official page shows naive examples.)
- **`add_to_date` output type** depends on flags—set `as_string`/`as_datetime` explicitly to avoid surprises.
- **PDF generation** needs wkhtmltopdf in path; CSS/JS support is limited. (The docs show usage; environment setup is on you.)
- **Email templates**: Jinja context keys in `args` must match template variables; attachments should use `file_url`.
- **Validation helpers** (`validate_*`) return/raise in different ways—decide early whether you want boolean checking or exception flow.

---

## Related Docs

- Database API (query data / pluck / permissions) — useful alongside utils.
- Developer API index — overview of Python API areas.
