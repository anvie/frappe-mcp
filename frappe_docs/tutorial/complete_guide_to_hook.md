# Complete Guide to `hooks.py`

The `hooks.py` file is the central configuration point for a Frappe app. It acts as the bridge between your custom application and the Frappe framework, allowing you to extend, override, and inject behavior without modifying core code.

This guide explains all major sections of `hooks.py`, their purpose, and how to use them—with practical examples.

---

## Location

Each Frappe app contains its own `hooks.py` at:

```
your_app/your_app/hooks.py
```

When Frappe starts, it loads hooks from all installed apps in order, applying overrides and merges where defined.

---

## Common Metadata

```python
app_name = "your_app"
app_title = "Your App"
app_publisher = "Your Name or Org"
app_description = "An app that extends Frappe"
app_icon = "octicon octicon-file-directory"
app_color = "blue"
app_email = "hello@example.com"
app_license = "MIT"
```

- **app_name**: Python package name (folder name)
- **app_title**: Human‑readable title
- **app_publisher**, **app_email**: Shown in app installer/info
- **app_icon**, **app_color**: Used in Desk UI
- **app_license**: Open source license string

---

## Includes: JS & CSS

You can inject global or per‑page assets.

```python
# Desk (admin) assets
desk_include_css = ["/assets/your_app/css/desk.css"]
desk_include_js  = ["/assets/your_app/js/desk.js"]

# Website (frontend) assets
web_include_css = ["/assets/your_app/css/site.css"]
web_include_js  = ["/assets/your_app/js/site.js"]

# Per‑doctype form includes
doctype_js = {
    "Sales Invoice": "public/js/sales_invoice.js",
    "User": "public/js/user.js"
}

# Per‑doctype listview includes
doctype_list_js = {
    "Sales Invoice": "public/js/sales_invoice_list.js"
}

# Per‑doctype treeview includes
doctype_tree_js = {
    "Account": "public/js/account_tree.js"
}

# Per‑doctype calendar includes
doctype_calendar_js = {
    "Task": "public/js/task_calendar.js"
}
```

---

## Fixtures

Fixtures are exported metadata (roles, custom fields, property setters, etc.) bundled with your app.

```python
fixtures = [
    "Custom Field",
    "Property Setter",
    {"dt": "Role", "filters": [["role_name", "in", ["Custom Role"]]]}
]
```

---

## Home Pages

Define what a user sees after login or at `/`.

```python
# For Desk
home_page = "login"

# For Website (when logged in)
role_home_page = {
    "Student": "/student-home",
    "Instructor": "/instructor-home"
}

# For Website (for guests)
website_home_page = "home"
```

---

## Generators

If you want a doctype to auto‑generate website pages (e.g., Blog Post).

```python
generated_doctypes = ["Blog Post", "Product"]
```

---

## Installation & Migration Hooks

Hooks that run automatically when app lifecycle events occur.

```python
# Before install
def before_install():
    pass

# After install
def after_install():
    pass

# Before uninstall
def before_uninstall():
    pass

# After uninstall
def after_uninstall():
    pass

# After app migrations
def after_migrate():
    pass
```

---

## Desk Notifications

Customize Desk notification count.

```python
# your_app/hooks.py
notification_config = "your_app.notifications.get_notification_config"
```

**`your_app/notifications.py`**

```python
def get_notification_config():
    return {
        "for_doctype": {
            "Task": {"assigned": "frappe.db.count('Task', {'status': 'Open'})"}
        }
    }
```

---

## Permission Hooks

Extend or override permission logic.

```python
doctype_js = {
    "Sales Invoice": "public/js/sales_invoice.js"
}

permission_query_conditions = {
    "Sales Invoice": "your_app.permissions.sales_invoice_conditions"
}

has_permission = {
    "Sales Invoice": "your_app.permissions.sales_invoice_has_permission"
}
```

---

## DocType Event Hooks

Run custom code during document lifecycle.

```python
doc_events = {
    "Sales Invoice": {
        "on_submit": "your_app.sales_invoice.on_submit",
        "validate": "your_app.sales_invoice.validate"
    },
    "*": {
        "on_update": "your_app.audit.log_change"
    }
}
```

Events available:

- `before_insert`, `after_insert`
- `validate`, `before_validate`
- `on_update`, `before_save`, `after_save`
- `before_submit`, `on_submit`, `on_cancel`, `on_trash`
- `after_delete`

---

## Scheduled Tasks

Automate recurring jobs.

```python
scheduler_events = {
    "all": [
        "your_app.tasks.all"
    ],
    "daily": [
        "your_app.tasks.daily"
    ],
    "hourly": [
        "your_app.tasks.hourly"
    ],
    "weekly": [
        "your_app.tasks.weekly"
    ],
    "monthly": [
        "your_app.tasks.monthly"
    ]
}
```

> Use `bench execute your_app.tasks.daily` to test a task.

---

## Testing Hooks

Define test behavior.

```python
# Run before tests
before_tests = "your_app.test_setup.setup"
```

---

## Override Methods & Classes

You can replace core Frappe methods or DocType classes with your own.

```python
# Override whitelisted methods
override_whitelisted_methods = {
    "frappe.desk.doctype.event.event.get_events": "your_app.event.get_events"
}

# Override doctype class
override_doctype_class = {
    "ToDo": "your_app.overrides.CustomToDo"
}
```

---

## Website Context & Templates

Modify global website context or provide custom templates.

```python
# Add context variables to all web pages
update_website_context = "your_app.website_context.update_context"

# Add custom Jinja methods
jinja = {
    "methods": ["your_app.utils.jinja_methods"],
    "filters": ["your_app.utils.jinja_filters"]
}

# Add web templates path
website_generators = ["Blog Post"]
```

---

## Override Doctype Dashboards

Customize dashboard charts and links.

```python
override_doctype_dashboards = {
    "Task": "your_app.task.dashboard.get_data"
}
```

---

## Authentication & Authorization Hooks

Control login flows.

```python
auth_hooks = [
    "your_app.auth.validate_login",
    "your_app.auth.post_login"
]

# Override user data fields
user_data_fields = [
    {
        "doctype": "ToDo",
        "filter_by": "owner",
        "redact_fields": ["description"],
        "partial": 1
    }
]
```

---

## Search & Query Overrides

Override search and link queries.

```python
override_doctype_class = {
    "Customer": "your_app.overrides.CustomCustomer"
}

override_doctype_dashboards = {
    "Customer": "your_app.customer_dashboard.get_data"
}
```

---

## Logging & Error Reporting

You can hook into error handling by defining methods in your app and configuring them here (advanced use).

---

## Putting It All Together

A minimal example:

```python
app_name = "my_app"
app_title = "My App"
app_publisher = "Me"
app_description = "Custom extensions"
app_email = "me@example.com"
app_license = "MIT"

web_include_js = ["/assets/my_app/js/web.js"]

doctype_js = {
    "Task": "public/js/task.js"
}

doc_events = {
    "Task": {
        "on_update": "my_app.task_hooks.on_update"
    }
}

scheduler_events = {
    "daily": ["my_app.tasks.daily"]
}

override_whitelisted_methods = {
    "frappe.desk.doctype.todo.todo.get_todos": "my_app.todo.get_todos"
}
```

---

## Best Practices

- Keep `hooks.py` **lean and declarative**. Put logic in separate modules.
- Avoid overriding core methods unless necessary.
- Document your hooks inside the file for maintainability.
- Test each hook (`bench execute`, `bench start`) before deploying.
